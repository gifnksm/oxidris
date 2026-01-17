use crossterm::event::Event;
use ratatui::Frame;

use crate::{
    command::play::screens::Screen,
    record::SessionHistory,
    schema::ai_model::AiModel,
    tui::{App, RenderMode, Tui},
};

const TICK_RATE: f64 = 60.0;
const FRAME_RATE: f64 = 60.0;

#[derive(Debug)]
pub struct PlayApp {
    screen: Screen,
}

impl PlayApp {
    pub fn manual(history_size: usize) -> Self {
        Self {
            screen: Screen::manual(TICK_RATE, history_size),
        }
    }

    pub fn auto(model: &AiModel, history_size: usize, turbo: bool) -> anyhow::Result<Self> {
        Ok(Self {
            screen: Screen::auto(TICK_RATE, model, history_size, turbo)?,
        })
    }

    pub fn into_history(self) -> SessionHistory {
        self.screen.into_history()
    }
}

impl App for PlayApp {
    fn init(&mut self, tui: &mut Tui) {
        tui.set_render_mode(RenderMode::throttled_from_rate(FRAME_RATE));
        tui.set_tick_rate(TICK_RATE);
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
