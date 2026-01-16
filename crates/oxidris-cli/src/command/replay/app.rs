use std::path::PathBuf;

use crossterm::event;
use ratatui::{DefaultTerminal, Frame};

use crate::{command::replay::app::screens::Screen, schema::record::RecordedSession};

mod screens;

#[derive(Debug)]
pub struct App {
    screen: Screen,
}

impl App {
    pub fn new(path: PathBuf, session: RecordedSession) -> Self {
        Self {
            screen: Screen::turn_viewer(path, session),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.screen.is_exiting() {
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        self.screen.draw(frame);
    }

    fn handle_events(&mut self) -> anyhow::Result<()> {
        let event = event::read()?;
        self.screen.handle_event(&event);
        Ok(())
    }
}
