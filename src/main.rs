mod synth;
mod effect;

use std::{collections::HashMap, f32::consts::SQRT_2, io, ops::Add, sync::mpsc::{self, channel, sync_channel, Receiver, Sender, SyncSender, TryRecvError}, thread};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::CrosstermBackend, style::Stylize, widgets::Paragraph, DefaultTerminal, Frame, Terminal};
use rodio::{OutputStream, Source};
use synth::{Envelope, Osc, Sound, Wave};

fn main() {
    let mut terminal = ratatui::init();    
    terminal.clear().unwrap();

    let mut app = App::new();

    app.run(&mut terminal).unwrap();

    ratatui::restore();
}


pub struct App {
    counter: u8,
    exit: bool,
}

impl App {
    fn new() -> Self {
        let app = App {
            counter: 0,
            exit: false,
        };


        return app;
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let (tx, rx) = mpsc::channel::<SynthCommand>();
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let synth = Test::new(rx);
        stream_handle.play_raw(synth.convert_samples()).unwrap_or(());

        while !self.exit {
            //terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&tx)?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let counter = self.counter;
        let string = format!("Hello! Counter is {counter}");
        let greeting = Paragraph::new(string)
            .white()
            .on_blue();
        frame.render_widget(greeting, frame.area());
    }

    fn handle_events(&mut self, tx: &Sender<SynthCommand>) -> io::Result<()> {
        match event::read().unwrap() {
            Event::Key(key) => {
                fn send_freq(f: usize, key: &KeyEvent, tx: &Sender<SynthCommand>) {
                    match key.kind {
                        KeyEventKind::Press => {tx.send((f, true)).unwrap()}
                        KeyEventKind::Release => {tx.send((f, false)).unwrap()}
                        _ => {}
                    }
                }

                match key.code {
                    KeyCode::Esc => {self.exit = true}
                    KeyCode::Char('a') => send_freq(1, &key, tx), // a
                    KeyCode::Char('w') => send_freq(2, &key, tx), // a#
                    KeyCode::Char('s') => send_freq(3, &key, tx), // b
                    KeyCode::Char('d') => send_freq(4, &key, tx), // c
                    KeyCode::Char('r') => send_freq(5, &key, tx), // c#
                    KeyCode::Char('f') => send_freq(6, &key, tx), // d
                    KeyCode::Char('t') => send_freq(7, &key, tx), // d#
                    KeyCode::Char('g') => send_freq(8, &key, tx), // e
                    KeyCode::Char('h') => send_freq(9, &key, tx), // f
                    KeyCode::Char('u') => send_freq(10, &key, tx), // f#
                    KeyCode::Char('j') => send_freq(11, &key, tx), // g
                    KeyCode::Char('i') => send_freq(12, &key, tx), // g#
                    KeyCode::Char('k') => send_freq(13, &key, tx), // a
                    KeyCode::Up => send_freq(100, &key, tx),
                    KeyCode::Down => send_freq(101, &key, tx),
                    KeyCode::Left => send_freq(102, &key, tx),
                    KeyCode::Right => send_freq(103, &key, tx),
                    _ => {}
                }

                return Ok(())
            }
            _ => return Ok(())
        };
    }
}

type SynthCommand = (usize, bool);

struct Test {
    trigger: Receiver<SynthCommand>,
    sounds: HashMap<usize, Box<dyn Sound>>,
    wave: Wave
}

impl Test {
    fn new(rx: Receiver<SynthCommand>) -> Self {
        return Self {
            trigger: rx,
            sounds: HashMap::new(),
            wave: Wave::Sine
        }
    }
}

impl Iterator for Test {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        //let base = 440.0 * (2.0 as f32).powf(1.0 / 12.0).powi(21); // mc donalds
        // let base = 440.0 * (2.0 as f32).powf(1.0 / 12.0).powi(16); // harry potter
        let base = 440.0 * (2.0 as f32).powf(1.0 / 12.0).powi(10); // omni man
        // let base = 440.0 * (2.0 as f32).powf(1.0 / 12.0).powi(8); // C
        fn semitones(freq: f32, n: usize) -> f32 {
            let factor = (2.0 as f32).powf(1.0/12.0);
            freq * factor.powi(n as i32)
        }

        match self.trigger.try_recv() {
            Ok((100, _)) => self.wave = Wave::Sine,
            Ok((101, _)) => self.wave = Wave::Saw,
            Ok((102, _)) => self.wave = Wave::Square,
            Ok((103, _)) => self.wave = Wave::Triangle,

            Ok((key, false)) => {
                if let Some(sound) = self.sounds.get_mut(&key) {
                    (*sound).stop()
                }
            },
            Ok((key, true)) => {
                /*if let Some(x) = self.sounds.get_mut(&key) {
                    x.wave = self.wave;
                    x.start()
                } else {
                 */
                    let mut osc = Osc::new(self.sample_rate(), semitones(base, key - 1), self.wave);
                    osc.connect(Box::new(Envelope::new(self.sample_rate(), 0.1, 0.05, 0.8, 0.25)));
                    osc.start();
                    self.sounds.insert(key, Box::new(osc));
                // }
            },
            Err(TryRecvError::Empty) => (),
            Err(_) => panic!("synth control channel disconnected")
        }

        let mut sample = self.sounds.iter_mut().fold(0.0, |sample, (_, sound)| {
            sample + sound.tick()
        });

        sample /= self.sounds.len() as f32;

        Some(sample)
    }
}

impl Source for Test {
    fn current_frame_len(&self) -> Option<usize> {
        return None
    }

    fn sample_rate(&self) -> u32 {
        48000
    }

    fn channels(&self) -> u16 {
        return 1
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        return None
    }
}