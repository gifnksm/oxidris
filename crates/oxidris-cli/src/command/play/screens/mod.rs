use crossterm::event::Event;
use oxidris_evaluator::turn_evaluator::TurnEvaluator;
use ratatui::Frame;

use crate::command::play::screens::{auto::AutoPlayScreen, manual::ManualPlayScreen};

pub mod auto;
pub mod manual;

#[derive(Debug)]
pub enum Screen {
    Manual(ManualPlayScreen),
    Auto(AutoPlayScreen),
}

impl Screen {
    pub fn manual(fps: u64) -> Self {
        Screen::Manual(ManualPlayScreen::new(fps))
    }

    pub fn auto(fps: u64, turn_evaluator: TurnEvaluator<'static>) -> Self {
        Screen::Auto(AutoPlayScreen::new(fps, turn_evaluator))
    }

    pub fn is_playing(&self) -> bool {
        match self {
            Screen::Manual(screen) => screen.is_playing(),
            Screen::Auto(screen) => screen.is_playing(),
        }
    }

    pub fn is_exiting(&self) -> bool {
        match self {
            Screen::Manual(screen) => screen.is_exiting(),
            Screen::Auto(screen) => screen.is_exiting(),
        }
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        match self {
            Screen::Manual(screen) => screen.draw(frame),
            Screen::Auto(screen) => screen.draw(frame),
        }
    }

    pub(crate) fn handle_event(&mut self, event: &Event) {
        match self {
            Screen::Manual(screen) => screen.handle_event(event),
            Screen::Auto(screen) => screen.handle_event(event),
        }
    }

    pub fn update_game(&mut self) {
        match self {
            Screen::Manual(screen) => screen.update_game(),
            Screen::Auto(screen) => screen.update_game(),
        }
    }
}
