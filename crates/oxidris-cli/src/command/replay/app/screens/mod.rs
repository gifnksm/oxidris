use std::path::PathBuf;

use crossterm::event::Event;
use ratatui::Frame;

use crate::{
    command::replay::app::screens::turn_viewer::TurnViewerScreen, schema::record::RecordedSession,
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

    pub fn is_exiting(&self) -> bool {
        match self {
            Screen::TurnViewer(screen) => screen.is_exiting(),
        }
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        match self {
            Screen::TurnViewer(screen) => screen.draw(frame),
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match self {
            Screen::TurnViewer(screen) => screen.handle_event(event),
        }
    }
}
