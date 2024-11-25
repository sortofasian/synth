use std::sync::mpsc::{Receiver, TryRecvError};

use rodio::Source;

use crate::nodes::{Envelope, Input, InputSlot, Node, Oscillator, Wave};

pub enum SynthCommand {
    Note(usize, bool),
    Wave(Wave),
}

pub struct Synth {
    trigger: Receiver<SynthCommand>,
    sounds: Vec<Box<Envelope>>,
}

impl Synth {
    pub fn new(rx: Receiver<SynthCommand>) -> Self {
        // let base = 440.0 * (2.0 as f32).powf(1.0 / 12.0).powi(21); // mc donalds
        // let base = 440.0 * (2.0 as f32).powf(1.0 / 12.0).powi(16); // harry potter
        // let base = 880.0 * (2.0 as f32).powf(1.0 / 12.0).powi(8); // C
        // let base = 880.0 * (2.0 as f32).powf(1.0 / 12.0).powi(10); // omni man
        let base = 110.0;
        fn semitones(freq: f32, n: usize) -> f32 {
            let factor = (2.0 as f32).powf(1.0 / 12.0);
            freq * factor.powi(n as i32)
        }

        return Self {
            trigger: rx,
            sounds: Vec::from_iter((0..17).map(|f| {
                let mut osc = Oscillator::new(48000);
                let freq = Input::Value(semitones(base, f));
                osc.set_input(Box::new(freq), InputSlot::Frequency).unwrap();

                let mut output = Envelope::new(48000, 0.05, 0.1, 0.2, 0.6);
                output
                    .set_input(Box::new(Input::Node(osc)), InputSlot::Input)
                    .unwrap();

                output
            })),
        };
    }
}

impl Iterator for Synth {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.trigger.try_recv() {
            Ok(SynthCommand::Wave(wave)) => {
                self.sounds.iter_mut().for_each(|output| {
                    if let Some(Input::Node(osc)) = output
                        .get_input(InputSlot::Input)
                        .unwrap()
                        .downcast_mut::<Input<f32>>()
                    {
                        osc.set_input(Box::new(Input::Value(wave)), InputSlot::Wave)
                            .unwrap();
                    }
                });
            }

            Ok(SynthCommand::Note(key, false)) => {
                if let Some(sound) = self.sounds.get_mut(key) {
                    (*sound).release()
                }
            }
            Ok(SynthCommand::Note(key, true)) => {
                if let Some(sound) = self.sounds.get_mut(key) {
                    (*sound).press();
                }
            }
            Err(TryRecvError::Empty) => (),
            Err(_) => panic!("synth control channel disconnected"),
        }

        let mut sample = self.sounds.iter_mut().fold(0.0, |sample, sound| {
            let sound: f32 = sound.tick().try_into().unwrap();
            sample + sound
        });

        sample /= self.sounds.len() as f32;

        Some(sample)
    }
}

impl Source for Synth {
    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }

    fn sample_rate(&self) -> u32 {
        48000
    }

    fn channels(&self) -> u16 {
        return 1;
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        return None;
    }
}
