use std::{f32::consts::PI, fs::File, i16};

const filter: &[f32] = &[
    0.023763740696978130,
    0.028369892700470929,
    0.032882947824167799,
    0.037213760056038943,
    0.041275300351152427,
    0.044984789636795010,
    0.048265744317032462,
    0.051049878768686600,
    0.053278812980215293,
    0.054905538635897727,
    0.055895603445089231,
    0.056227981174950710,
    0.055895603445089231,
    0.054905538635897727,
    0.053278812980215293,
    0.051049878768686600,
    0.048265744317032462,
    0.044984789636795010,
    0.041275300351152427,
    0.037213760056038943,
    0.032882947824167799,
    0.028369892700470929,
    0.023763740696978130,
];

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SampleRate, SizedSample,
};

fn main() {
    let device = cpal::default_host().default_output_device().unwrap();

    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::I8 => make_stream::<i8>(device, config.into()),
        cpal::SampleFormat::U8 => make_stream::<u8>(device, config.into()),
        cpal::SampleFormat::I16 => make_stream::<i16>(device, config.into()),
        cpal::SampleFormat::U16 => make_stream::<u16>(device, config.into()),
        cpal::SampleFormat::I32 => make_stream::<i32>(device, config.into()),
        cpal::SampleFormat::U32 => make_stream::<u32>(device, config.into()),
        cpal::SampleFormat::I64 => make_stream::<i64>(device, config.into()),
        cpal::SampleFormat::U64 => make_stream::<u64>(device, config.into()),
        cpal::SampleFormat::F32 => make_stream::<f32>(device, config.into()),
        cpal::SampleFormat::F64 => make_stream::<f64>(device, config.into()),
        _ => panic!(),
    }
}

fn make_stream<T>(device: cpal::Device, config: cpal::StreamConfig)
where
    T: SizedSample + FromSample<f32>,
{
    let mut osc = Sine::new(config.sample_rate.0, 440.0);
    let mut lfo = Sine::new(config.sample_rate.0, 40.0);
    let mut env = Envelope::new(config.sample_rate.0, 0.01, 0.06, 0.6, 0.7);
    let rate = config.sample_rate.0;
    let press_ticks = (rate as f32 * 0.1) as u32;
    let measure_samps = rate;
    let quarter_samps = (measure_samps as f32 * 0.33) as u32;
    let mut ticks = 0;
    let mut sounds = vec![0.0; filter.len()];

    let mut file = File::open("./guitar.wav").unwrap();
    let (header, data) = wav::read(&mut file).unwrap();
    let mut sample = data.try_into_sixteen().unwrap();
    println!("{header:#?}");
    let max_val = sample.iter().cloned().fold(0, i16::max) as f32 / i16::MAX as f32;

    let stream = device
        .build_output_stream(
            &config,
            move |output: &mut [T], _| {
                for frame in output.chunks_mut(config.channels.into()) {
                    if ticks % measure_samps == 0 {
                        osc.freq = 440.0;
                        env.press();
                    }

                    /*
                    if (ticks - press_ticks) % measure_samps == 0 {
                        env.release();
                    }


                    if (ticks - quarter_samps) % measure_samps == 0 {
                        osc.freq = 440.0 * (2.0 as f32).powf(1.0 / 12.0).powi(5);
                        env.press();
                    }
                    if (ticks - press_ticks - quarter_samps) % measure_samps == 0 {
                        env.release();
                    }
                    */

                    /*
                    if (ticks - (quarter_samps * 2)) % measure_samps == 0 {
                        osc.freq = 440.0 * (2.0 as f32).powf(1.0 / 12.0).powi(8);
                        env.press();
                    }
                    if (ticks - press_ticks - (quarter_samps * 2)) % measure_samps == 0 {
                        env.release();
                    }
                    */

                    ticks += 1;

                    // sine
                    let mut sound = osc.tick();

                    // envelope
                    // sound *= env.tick();

                    //let sound = sound * (1.0 - (lfo.tick() / 5.0));
                    sound *= 0.2;

                    sound = match sample.get((ticks as f32) as usize) {
                        Some(s) => *s as f32 / i16::MAX as f32,
                        None => 0.0,
                    };

                    // sound = if 0.001 < sound { sound } else { 0.0 };
                    sound = sound.powi(3) * 3.0;

                    sound = (sound + sounds.last().unwrap()) / 2.0;
                    sounds.push(sound);

                    let value = T::from_sample(sound);
                    for sample in frame.iter_mut() {
                        *sample = value
                    }
                }
            },
            |_| panic!(),
            None,
        )
        .unwrap();

    stream.play().unwrap();
    loop {}
}

struct Sine {
    rate: u32,
    ticks: u32,
    freq: f32,
}

impl Sine {
    fn new(rate: u32, freq: f32) -> Self {
        Sine {
            rate,
            ticks: 0,
            freq,
        }
    }

    fn tick(&mut self) -> f32 {
        if self.freq < 1.0 {
            if (self.ticks as f32 / self.rate as f32) >= (1.0 / self.freq) {
                self.ticks = 0;
            }
        } else if self.ticks == self.rate {
            self.ticks = 0;
        }

        let pos = self.ticks as f32 / self.rate as f32;
        let val = f32::cos(pos * 2.0 * PI * self.freq as f32);

        self.ticks += 1;
        return val;
    }
}

struct Envelope {
    ticks: u32,
    state: EnvelopeState,
    last_val: f32,

    sustain: f32,
    attack_samps: u32,
    decay_samps: u32,
    release_samps: u32,
}

#[derive(Debug)]
enum EnvelopeState {
    Idle,
    Attack(f32),
    Decay,
    Sustain,
    Release(f32),
}

impl Envelope {
    fn new(rate: u32, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        return Envelope {
            ticks: 0,
            state: EnvelopeState::Idle,
            last_val: 0.0,
            sustain,
            attack_samps: (attack * rate as f32) as u32,
            decay_samps: (decay * rate as f32) as u32,
            release_samps: (release * rate as f32) as u32,
        };
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

    fn press(&mut self) {
        self.ticks = 0;
        self.state = EnvelopeState::Attack(self.last_val);
    }

    fn release(&mut self) {
        self.ticks = 0;
        self.state = EnvelopeState::Release(self.last_val);
    }
}
