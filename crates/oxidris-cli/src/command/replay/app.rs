use std::path::PathBuf;

use crossterm::event::Event;
use ratatui::Frame;

use crate::{
    command::replay::screens::Screen,
    schema::record::RecordedSession,
    tui::{App, Tui},
};

#[derive(Debug)]
pub struct ReplayApp {
    screen: Screen,
}

impl ReplayApp {
    pub fn new(path: PathBuf, session: RecordedSession) -> Self {
        Self {
            screen: Screen::turn_viewer(path, session),
        }
    }
}

impl App for ReplayApp {
    fn init(&mut self, _tui: &mut Tui) {}

    fn should_exit(&self) -> bool {
        self.screen.should_exit()
    }

    fn handle_event(&mut self, tui: &mut Tui, event: Event) {
        self.screen.handle_event(tui, &event);
    }

    fn draw(&self, frame: &mut Frame) {
        self.screen.draw(frame);
    }

    fn update(&mut self, _tui: &mut Tui) {
        self.screen.update();
    }
}
