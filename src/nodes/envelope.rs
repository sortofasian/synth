use std::{any::Any, mem};

use super::{into_input, Input, InputSlot, Node};

pub enum EnvelopeState {
    Idle,
    Attack(f32),
    Decay,
    Sustain,
    Release(f32),
}
pub struct Envelope {
    input: Input<f32>,
    level: f32,
    ticks: u32,
    state: EnvelopeState,

    attack_samps: u32,
    decay_samps: u32,
    sustain: f32,
    release_samps: u32,
}

impl Envelope {
    pub fn new(rate: u32, attack: f32, decay: f32, sustain: f32, release: f32) -> Box<Self> {
        let obj = Envelope {
            input: Input::Value(0.0),
            level: 0.0,
            ticks: 0,
            state: EnvelopeState::Idle,

            sustain,
            attack_samps: (attack * rate as f32) as u32,
            decay_samps: (decay * rate as f32) as u32,
            release_samps: (release * rate as f32) as u32,
        };

        Box::new(obj)
    }
}

impl Node<f32> for Envelope {
    fn press(&mut self) {
        if let Input::Node(node) = &mut self.input {
            node.press();
        }

        self.ticks = 0;
        self.state = EnvelopeState::Attack(self.level);
    }

    fn release(&mut self) {
        self.ticks = 0;
        self.state = EnvelopeState::Release(self.level);
    }

    fn tick(&mut self) -> f32 {
        let val = match self.state {
            EnvelopeState::Idle => {
                self.ticks = 0;

                if let Input::Node(node) = &mut self.input {
                    node.release();
                }

                0.0
            }
            EnvelopeState::Attack(start_val) => {
                self.ticks += 1;

                if self.ticks >= self.attack_samps {
                    self.ticks = 0;
                    self.state = EnvelopeState::Decay;
                    1.0
                } else {
                    let pos = self.ticks as f32 / self.attack_samps as f32;
                    pos * (1.0 - start_val) + start_val
                }
            }
            EnvelopeState::Decay => {
                self.ticks += 1;

                if self.ticks >= self.decay_samps {
                    self.ticks = 0;
                    self.state = EnvelopeState::Sustain;
                    return self.sustain;
                }

                let pos = self.ticks as f32 / self.decay_samps as f32;
                1.0 - (pos * (1.0 - self.sustain))
            }
            EnvelopeState::Sustain => {
                self.ticks = 0;
                self.sustain
            }
            EnvelopeState::Release(start_val) => {
                self.ticks += 1;

                if self.ticks >= self.release_samps || start_val == 0.0 {
                    self.state = EnvelopeState::Idle;
                    0.0
                } else {
                    let pos = self.ticks as f32 / self.release_samps as f32;
                    (1.0 - pos) * start_val
                }
            }
        };

        self.level = val;

        let sample: f32 = self.input.value().try_into().unwrap();
        return sample * self.level;
    }

    fn get_input(&mut self, slot: InputSlot) -> Result<&mut dyn Any, ()> {
        match slot {
            InputSlot::Input => Ok(&mut self.input),
            _ => Err(()),
        }
    }

    fn set_input(&mut self, input: Box<dyn Any>, slot: InputSlot) -> Result<Box<dyn Any>, ()> {
        match slot {
            InputSlot::Input => {
                let input = match into_input(input) {
                    Ok(input) => input,
                    Err(_) => return Err(()),
                };

                Ok(Box::new(mem::replace(&mut self.input, input)))
            }
            _ => Err(()),
        }
    }
}
