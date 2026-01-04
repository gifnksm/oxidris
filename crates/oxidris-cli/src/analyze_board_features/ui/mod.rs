use oxidris_evaluator::board_feature::BoxedBoardFeature;

use self::app::App;

use super::{
    data::{BoardFeatureStatistics, BoardSample},
    index::BoardIndex,
};

mod app;
mod screens;

pub(crate) fn run_tui(
    features: Vec<BoxedBoardFeature>,
    board_samples: Vec<BoardSample>,
    statistics: Vec<BoardFeatureStatistics>,
    board_index: BoardIndex,
) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new(features, board_samples, statistics, board_index).run(&mut terminal);
    ratatui::restore();
    app_result
}
