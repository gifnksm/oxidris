use oxidris_evaluator::board_feature::ALL_BOARD_FEATURES;

use self::app::App;

use super::{
    data::{BoardFeatureStatistics, BoardSample},
    index::BoardIndex,
};

mod app;
mod screens;

pub(crate) fn run_tui(
    board_samples: Vec<BoardSample>,
    statistics: [BoardFeatureStatistics; ALL_BOARD_FEATURES.len()],
    board_index: BoardIndex,
) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new(board_samples, statistics, board_index).run(&mut terminal);
    ratatui::restore();
    app_result
}
