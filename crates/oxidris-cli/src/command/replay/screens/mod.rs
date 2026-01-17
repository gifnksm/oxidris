use std::path::PathBuf;

use crossterm::event::Event;
use ratatui::Frame;

use crate::{
    command::replay::screens::turn_viewer::TurnViewerScreen, schema::record::RecordedSession,
    tui::Tui,
};

mod turn_viewer;

#[derive(Debug)]
pub enum Screen {
    TurnViewer(TurnViewerScreen),
}

impl Screen {
    pub fn turn_viewer(path: PathBuf, session: RecordedSession) -> Self {
        Self::TurnViewer(TurnViewerScreen::new(path, session))
    }

    pub fn should_exit(&self) -> bool {
        match self {
            Screen::TurnViewer(screen) => screen.should_exit(),
        }
    }

    pub fn handle_event(&mut self, tui: &mut Tui, event: &Event) {
        match self {
            Screen::TurnViewer(screen) => screen.handle_event(tui, event),
        }
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        match self {
            Screen::TurnViewer(screen) => screen.draw(frame),
        }
    }

    pub fn update(&mut self) {
        match self {
            Screen::TurnViewer(screen) => screen.update(),
        }
    }
}
