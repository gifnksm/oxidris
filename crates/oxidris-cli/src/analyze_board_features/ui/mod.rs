use oxidris_ai::board_feature::ALL_BOARD_FEATURES;

use self::app::App;

use super::{
    data::{BoardFeatures, FeatureStatistics},
    index::BoardIndex,
};

mod app;
mod screens;

pub(crate) fn run_tui(
    boards_features: Vec<BoardFeatures>,
    statistics: [FeatureStatistics; ALL_BOARD_FEATURES.len()],
    board_index: BoardIndex,
) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new(boards_features, statistics, board_index).run(&mut terminal);
    ratatui::restore();
    app_result
}
