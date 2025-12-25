use crossterm::event::{self, Event, KeyEventKind};
use oxidris_ai::ALL_METRICS;
use ratatui::{DefaultTerminal, Frame};

use crate::tune_metrics::{
    data::{BoardMetrics, MetricStatistics},
    index::BoardIndex,
};

use super::screens::metrics_list::MetricsListScreen;

#[derive(Debug)]
pub struct App {
    data: AppData,
    screen: Screen,
    metrics_list_screen: MetricsListScreen,
}

#[derive(Debug)]
pub struct AppData {
    pub boards_metrics: Vec<BoardMetrics>,
    pub statistics: [MetricStatistics; ALL_METRICS.len()],
    pub board_index: BoardIndex,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    MetricsList,
    Exiting,
}

impl App {
    pub fn new(
        boards_metrics: Vec<BoardMetrics>,
        statistics: [MetricStatistics; ALL_METRICS.len()],
        board_index: BoardIndex,
    ) -> Self {
        let data = AppData {
            boards_metrics,
            statistics,
            board_index,
        };
        Self {
            data,
            screen: Screen::default(),
            metrics_list_screen: MetricsListScreen::default(),
        }
    }

    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while self.screen != Screen::Exiting {
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match self.screen {
            Screen::MetricsList => {
                self.metrics_list_screen.draw(frame, &self.data);
            }
            Screen::Exiting => { /* Nothing to draw */ }
        }
    }

    fn handle_events(&mut self) -> anyhow::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match self.screen {
            Screen::MetricsList => {
                self.metrics_list_screen
                    .handle_input(key_event, &mut self.screen);
            }
            Screen::Exiting => { /* No input handling when exiting */ }
        }
    }
}
