use super::{Effect, Sound};

pub struct Osc {
    rate: u32,
    pos: f32,
    tick: u32,
    run: bool,
    pub freq: f32,
    effect: Option<Box<dyn Effect>>,
    pub wave: Wave,
}

#[derive(Debug, Clone, Copy)]
pub enum Wave {
    Sine,
    Square,
    Saw,
    Triangle,
}

impl Osc {
    pub fn new(rate: u32, freq: f32, wave: Wave) -> Self {
        Osc {
            rate,
            run: false,
            tick: 0,
            pos: 0.0,
            freq,
            effect: None,
            wave
        }
    }
}

impl Sound for Osc {
    fn start(&mut self) {
        if let Some(effect) = self.effect.as_mut() {
            effect.start();
        }

        self.tick = 0;
        self.run = true;
    }

    fn stop(&mut self) {
        if let Some(effect) = self.effect.as_mut() {
            effect.stop();
        }

        self.run = false;
    }

    fn connect(&mut self, dest: Box<dyn Effect>) -> &mut Box<dyn Effect> {
        self.effect = Some(dest);
        return self.effect.as_mut().unwrap();
    }

    fn disconnect(&mut self) -> Option<Box<dyn Effect>> {
        return self.effect.take()
    }

    fn tick(&mut self) -> f32 {
        /* if self.run == false && self.pos >= 1.0 {
            let diff = self.pos - 1.0;
            return diff;
        } */

        if self.pos >= 1.0 {
            self.pos -= 1.0;
        }

        self.pos += self.freq / self.rate as f32;

        let max_harmonic = (self.rate as f32 / 2.0 / self.freq) as usize;

        let mut sample = match self.wave {
            Wave::Sine => sine(self.pos),
            Wave::Saw => saw(self.pos, false, max_harmonic),
            Wave::Square => square(self.pos, max_harmonic),
            Wave::Triangle => triangle(self.pos, max_harmonic)
        };

        if let Some(effect) = self.effect.as_mut() {
            sample = effect.process(sample);
        }

        // TODO: remove once ui can get key up events
        self.tick += 1;
        if self.tick > self.rate / 10 {
            self.stop();
        }

        return sample;
    }
}

fn sine(pos: f32) -> f32 {
    (pos * 2.0 * std::f32::consts::PI).sin()
}

fn saw(pos: f32, ramp_up: bool, max_harmonic: usize) -> f32 {
    (1..max_harmonic).fold(sine(pos), |sample, i| {
        let n = i as f32;
        sample + (sine(pos * n) / (n * 2.0))
    })
}

fn triangle(pos: f32, max_harmonic: usize) -> f32 {
    (1..(max_harmonic / 2)).fold(sine(pos), |sample, i| {
        let n = (2 * i - 1) as f32;
        (sample + (sine(pos * n) / n.powi(2))) * (-1.0 as f32).powi(i as i32)
    })
}

fn square(pos: f32, max_harmonic: usize) -> f32 {
    (1..(max_harmonic / 2)).fold(sine(pos), |sample, i| {
        let n = (2 * i - 1) as f32;
        sample + sine(pos * n) / n
    })
}