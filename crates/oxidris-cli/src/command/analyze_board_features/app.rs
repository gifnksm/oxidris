use crossterm::event::Event;
use oxidris_analysis::{
    index::BoardIndex, sample::BoardSample, statistics::BoardFeatureStatistics,
};
use oxidris_evaluator::board_feature::BoxedBoardFeature;
use ratatui::Frame;

use crate::{
    command::analyze_board_features::screens::Screen,
    tui::{App, Tui},
};

#[derive(Debug)]
pub struct AnalyzeBoardApp {
    screen: Screen,
}

#[derive(Debug)]
pub struct AppData {
    pub board_samples: Vec<BoardSample>,
    pub statistics: Vec<BoardFeatureStatistics>,
    #[expect(unused, reason = "may be used later")] // TODO
    pub board_index: BoardIndex,
}

impl AnalyzeBoardApp {
    pub fn new(
        features: Vec<BoxedBoardFeature>,
        board_samples: Vec<BoardSample>,
        statistics: Vec<BoardFeatureStatistics>,
        board_index: BoardIndex,
    ) -> Self {
        let data = AppData {
            board_samples,
            statistics,
            board_index,
        };
        Self {
            screen: Screen::feature_list(data, features),
        }
    }
}

impl App for AnalyzeBoardApp {
    fn init(&mut self, _tui: &mut Tui) {}

    fn should_exit(&self) -> bool {
        self.screen.should_exit()
    }

    fn handle_event(&mut self, _tui: &mut Tui, event: Event) {
        self.screen.handle_event(&event);
    }

    fn draw(&self, frame: &mut Frame) {
        self.screen.draw(frame);
    }

    fn update(&mut self, _tui: &mut Tui) {}
}
