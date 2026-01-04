use crossterm::event::{self, Event, KeyEventKind};
use oxidris_evaluator::board_feature::ALL_BOARD_FEATURES;
use ratatui::{DefaultTerminal, Frame};

use crate::analyze_board_features::{
    data::{BoardFeatureStatistics, BoardSample},
    index::BoardIndex,
};

use super::screens::feature_list::FeatureListScreen;

#[derive(Debug)]
pub struct App {
    data: AppData,
    screen: Screen,
    features_list_screen: FeatureListScreen,
}

#[derive(Debug)]
pub struct AppData {
    pub board_samples: Vec<BoardSample>,
    pub statistics: [BoardFeatureStatistics; ALL_BOARD_FEATURES.len()],
    pub board_index: BoardIndex,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    FeaturesList,
    Exiting,
}

impl App {
    pub fn new(
        board_samples: Vec<BoardSample>,
        statistics: [BoardFeatureStatistics; ALL_BOARD_FEATURES.len()],
        board_index: BoardIndex,
    ) -> Self {
        let data = AppData {
            board_samples,
            statistics,
            board_index,
        };
        Self {
            data,
            screen: Screen::default(),
            features_list_screen: FeatureListScreen::default(),
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
            Screen::FeaturesList => {
                self.features_list_screen.draw(frame, &self.data);
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
            Screen::FeaturesList => {
                self.features_list_screen
                    .handle_input(key_event, &mut self.screen);
            }
            Screen::Exiting => { /* No input handling when exiting */ }
        }
    }
}
