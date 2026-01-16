use crossterm::event::Event;
use oxidris_evaluator::board_feature::BoxedBoardFeature;
use ratatui::Frame;

use crate::command::analyze_board_features::{
    app::AppData, screens::feature_list::FeatureListScreen,
};

mod feature_list;

#[derive(Debug)]
pub enum Screen {
    FeatureList(FeatureListScreen),
}

impl Screen {
    #[must_use]
    pub fn feature_list(data: AppData, features: Vec<BoxedBoardFeature>) -> Self {
        Self::FeatureList(FeatureListScreen::new(data, features))
    }

    pub fn should_exit(&self) -> bool {
        match self {
            Self::FeatureList(screen) => screen.should_exit(),
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        match self {
            Self::FeatureList(screen) => screen.handle_event(event),
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        match self {
            Self::FeatureList(screen) => screen.draw(frame),
        }
    }
}
