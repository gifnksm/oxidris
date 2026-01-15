use crossterm::event::Event;
use ratatui::Frame;

use crate::{
    command::play::screens::{auto::AutoPlayScreen, manual::ManualPlayScreen},
    record::SessionHistory,
    schema::ai_model::AiModel,
};

pub mod auto;
pub mod manual;

#[derive(Debug)]
pub enum Screen {
    Manual(ManualPlayScreen),
    Auto(AutoPlayScreen),
}

impl Screen {
    pub fn manual(fps: u64, history_size: usize) -> Self {
        Screen::Manual(ManualPlayScreen::new(fps, history_size))
    }

    pub fn auto(
        fps: u64,
        model: &AiModel,
        history_size: usize,
        turbo: bool,
    ) -> anyhow::Result<Self> {
        let screen = AutoPlayScreen::new(fps, model, history_size, turbo)?;
        Ok(Screen::Auto(screen))
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

    pub fn into_history(self) -> SessionHistory {
        match self {
            Screen::Manual(screen) => screen.into_history(),
            Screen::Auto(screen) => screen.into_history(),
        }
    }
}
