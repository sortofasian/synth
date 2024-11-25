use super::{into_input, types::Wave, Input, InputSlot, Node};
use std::{any::Any, mem};

pub struct Oscillator {
    rate: u32,
    pos: f32,
    run: bool,

    pub freq: Input<f32>,
    pub amp: Input<f32>,

    pub wave: Input<Wave>,
}

impl Oscillator {
    pub fn new(rate: u32) -> Box<Self> {
        let obj = Self {
            rate,
            pos: 0.0,
            run: false,
            amp: Input::Value(1.0),
            freq: Input::Value(440.0),
            wave: Input::Value(Wave::Sine),
        };

        Box::new(obj)
    }
    fn synthesize(&mut self, freq: f32, amp: f32) -> f32 {
        let max_harmonic = (self.rate as f32 / 2.0 / freq) as usize;

        let sample = match self.wave.value().try_into().unwrap() {
            Wave::Sine => Self::sine(self.pos),
            Wave::Saw => Self::saw(self.pos, false, max_harmonic),
            Wave::Square => Self::square(self.pos, max_harmonic),
            Wave::Triangle => Self::triangle(self.pos, max_harmonic),
        };
        sample * amp
    }

    fn sine(pos: f32) -> f32 {
        (pos * 2.0 * std::f32::consts::PI).sin()
    }

    fn saw(pos: f32, ramp_up: bool, max_harmonic: usize) -> f32 {
        let sample = (1..max_harmonic).fold(Self::sine(pos), |sample, i| {
            let n = i as f32;
            sample + (Self::sine(pos * n) / (n * 2.0))
        });

        match ramp_up {
            true => sample * -1.0,
            false => sample,
        }
    }

    fn triangle(pos: f32, max_harmonic: usize) -> f32 {
        (1..(max_harmonic / 2)).fold(Self::sine(pos), |sample, i| {
            let n = (2 * i - 1) as f32;
            (sample + (Self::sine(pos * n) / n.powi(2))) * (-1.0 as f32).powi(i as i32)
        })
    }

    fn square(pos: f32, max_harmonic: usize) -> f32 {
        (1..(max_harmonic / 2)).fold(Self::sine(pos), |sample, i| {
            let n = (2 * i - 1) as f32;
            sample + Self::sine(pos * n) / n
        })
    }
}

impl Node<f32> for Oscillator {
    fn set_input(&mut self, input: Box<dyn Any>, slot: InputSlot) -> Result<Box<dyn Any>, ()> {
        match slot {
            InputSlot::Amplitude => {
                if let Ok(input) = into_input(input) {
                    return Ok(Box::new(mem::replace(&mut self.amp, input)));
                }
            }
            InputSlot::Frequency => {
                if let Ok(input) = into_input(input) {
                    return Ok(Box::new(mem::replace(&mut self.freq, input)));
                }
            }
            InputSlot::Wave => {
                if let Ok(input) = into_input(input) {
                    return Ok(Box::new(mem::replace(&mut self.wave, input)));
                }
            }
            _ => return Err(()),
        };
        return Err(());
    }

    fn get_input(&mut self, slot: InputSlot) -> Result<&mut dyn Any, ()> {
        match slot {
            InputSlot::Amplitude => Ok(&mut self.amp),
            InputSlot::Frequency => Ok(&mut self.freq),
            InputSlot::Wave => Ok(&mut self.wave),
            _ => Err(()),
        }
    }

    fn tick(&mut self) -> f32 {
        let freq: f32 = self.freq.value().try_into().unwrap();
        let amp: f32 = self.amp.value().try_into().unwrap();

        if self.pos >= 1.0 {
            self.pos -= 1.0;
        }

        if self.run == true {
            self.pos += freq / self.rate as f32;
        }

        self.synthesize(freq, amp)
    }

    fn press(&mut self) {
        self.run = true;
    }

    fn release(&mut self) {
        self.run = false;
    }
}
