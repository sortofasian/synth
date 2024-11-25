mod nodes;
mod synth;

use std::{
    io,
    sync::mpsc::{self, Sender},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use nodes::Wave;
use ratatui::{style::Stylize, widgets::Paragraph, DefaultTerminal, Frame};
use rodio::{OutputStream, Source};

use synth::{Synth, SynthCommand};

fn main() {
    let mut terminal = ratatui::init();
    terminal.clear().unwrap();

    let mut app = App::new();

    app.run(&mut terminal).unwrap();

    ratatui::restore();
}

pub struct App {
    exit: bool,
}

impl App {
    fn new() -> Self {
        let app = App { exit: false };

        return app;
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let (tx, rx) = mpsc::channel::<SynthCommand>();
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let synth = Synth::new(rx);
        stream_handle
            .play_raw(synth.convert_samples())
            .unwrap_or(());

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&tx)?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let greeting = Paragraph::new("").white().on_blue();
        frame.render_widget(greeting, frame.area());
    }

    fn handle_events(&mut self, tx: &Sender<SynthCommand>) -> io::Result<()> {
        match event::read().unwrap() {
            Event::Key(key) => {
                fn send_freq(f: usize, key: &KeyEvent, tx: &Sender<SynthCommand>) {
                    match key.kind {
                        KeyEventKind::Press => tx.send(SynthCommand::Note(f, true)).unwrap(),
                        KeyEventKind::Release => tx.send(SynthCommand::Note(f, false)).unwrap(),
                        _ => {}
                    }
                }

                match key.code {
                    KeyCode::Esc => self.exit = true,
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
                    KeyCode::Char('o') => send_freq(14, &key, tx), // a#
                    KeyCode::Char('l') => send_freq(15, &key, tx), // b
                    KeyCode::Char(';') => send_freq(16, &key, tx), // c
                    KeyCode::Up => tx.send(SynthCommand::Wave(Wave::Sine)).unwrap(),
                    KeyCode::Down => tx.send(SynthCommand::Wave(Wave::Saw)).unwrap(),
                    KeyCode::Left => tx.send(SynthCommand::Wave(Wave::Square)).unwrap(),
                    KeyCode::Right => tx.send(SynthCommand::Wave(Wave::Triangle)).unwrap(),
                    _ => {}
                }

                return Ok(());
            }
            _ => return Ok(()),
        };
    }
}
