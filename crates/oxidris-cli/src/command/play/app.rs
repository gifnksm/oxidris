use crossterm::event::Event;
use ratatui::Frame;

use crate::{
    command::play::screens::Screen,
    record::SessionHistory,
    schema::ai_model::AiModel,
    tui::{App, Tui},
};

const FPS: u64 = 60;

#[derive(Debug)]
pub struct PlayApp {
    screen: Screen,
}

impl PlayApp {
    pub fn manual(history_size: usize) -> Self {
        Self {
            screen: Screen::manual(FPS, history_size),
        }
    }

    pub fn auto(model: &AiModel, history_size: usize, turbo: bool) -> anyhow::Result<Self> {
        Ok(Self {
            screen: Screen::auto(FPS, model, history_size, turbo)?,
        })
    }

    pub fn into_history(self) -> SessionHistory {
        self.screen.into_history()
    }
}

impl App for PlayApp {
    #[expect(clippy::cast_precision_loss)]
    fn init(&mut self, tui: &mut Tui) {
        tui.set_frame_rate(FPS as f64);
        tui.set_tick_rate(FPS as f64);
    }

    fn should_exit(&self) -> bool {
        self.screen.should_exit()
    }

    fn handle_event(&mut self, _tui: &mut Tui, event: Event) {
        self.screen.handle_event(&event);
    }

    fn draw(&self, frame: &mut Frame) {
        self.screen.draw(frame);
    }

    fn update(&mut self, _tui: &mut Tui) {
        if self.screen.is_playing() {
            self.screen.update();
        }
    }
}
