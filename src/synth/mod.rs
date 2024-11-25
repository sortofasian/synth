mod osc;

pub use osc::{Osc, Wave};

pub trait Sound : Send {
    fn tick(&mut self) -> f32;
    fn start(&mut self);
    fn stop(&mut self);
    fn connect(&mut self, dest: Box<dyn Effect>) -> &mut Box<dyn Effect>;
    fn disconnect(&mut self) -> Option<Box<dyn Effect>>;
}

pub trait Effect : Sound {
    fn process(&mut self, sample: f32) -> f32;
}

pub enum EnvelopeState {
    Idle,
    Attack(f32),
    Decay,
    Sustain,
    Release(f32),
}
pub struct Envelope {
    effect: Option<Box<dyn Effect>>,
    last_val: f32,
    ticks: u32,
    state: EnvelopeState,

    attack_samps: u32,
    decay_samps: u32,
    sustain: f32,
    release_samps: u32
}

impl Envelope {
    pub fn new(rate: u32, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        return Envelope {
            ticks: 0,
            state: EnvelopeState::Idle,
            last_val: 0.0,
            sustain,
            attack_samps: (attack * rate as f32) as u32,
            decay_samps: (decay * rate as f32) as u32,
            release_samps: (release * rate as f32) as u32,
            effect: None
        };
    }
}

impl Effect for Envelope {
    fn process(&mut self, sample: f32) -> f32 {
        let val = self.tick();
        sample * val
    }
}

impl Sound for Envelope {
    fn connect(&mut self, dest: Box<dyn Effect>) -> &mut Box<dyn Effect> {
        self.effect = Some(dest);
        return self.effect.as_mut().unwrap();
    }

    fn disconnect(&mut self) -> Option<Box<dyn Effect>> {
        self.effect.take()
    }

    fn start(&mut self) {
        if let Some(effect) = self.effect.as_mut() {
            effect.start();
        }

        self.ticks = 0;
        self.state = EnvelopeState::Attack(self.last_val);
    }

    fn stop(&mut self) {
        if let Some(effect) = self.effect.as_mut() {
            effect.stop();
        }

        self.ticks = 0;
        self.state = EnvelopeState::Release(self.last_val);
    }

    fn tick(&mut self) -> f32 {
        let val = match self.state {
            EnvelopeState::Idle => {
                self.ticks = 0;
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

        self.last_val = val;

        return val;
    }
}