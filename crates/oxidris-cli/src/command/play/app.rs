use std::{
    thread,
    time::{Duration, Instant},
};

use crossterm::event;
use ratatui::{DefaultTerminal, Frame};

use crate::{command::play::screens::Screen, record::SessionHistory, schema::ai_model::AiModel};

const FPS: u64 = 60;

#[derive(Debug)]
pub struct App {
    screen: Screen,
}

impl App {
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

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        let tick_rate = Duration::from_secs(1) / u32::try_from(FPS).unwrap();

        while !self.screen.is_exiting() {
            let now = Instant::now();
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;

            if self.screen.is_playing() {
                self.update_game();
            }

            let elapsed = now.elapsed();
            if let Some(rest) = tick_rate.checked_sub(elapsed) {
                thread::sleep(rest);
            }
        }
        Ok(())
    }

    pub fn into_history(self) -> SessionHistory {
        self.screen.into_history()
    }

    fn draw(&self, frame: &mut Frame<'_>) {
        self.screen.draw(frame);
    }

    fn handle_events(&mut self) -> anyhow::Result<()> {
        while event::poll(Duration::ZERO)? {
            let event = event::read()?;
            self.screen.handle_event(&event);
        }
        Ok(())
    }

    fn update_game(&mut self) {
        self.screen.update_game();
    }
}
