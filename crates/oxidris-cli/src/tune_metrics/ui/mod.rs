use oxidris_ai::ALL_METRICS;

use self::app::App;

use super::{
    data::{BoardMetrics, MetricStatistics},
    index::BoardIndex,
};

mod app;
mod screens;

pub(crate) fn run_tui(
    boards_metrics: Vec<BoardMetrics>,
    statistics: [MetricStatistics; ALL_METRICS.len()],
    board_index: BoardIndex,
) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new(boards_metrics, statistics, board_index).run(&mut terminal);
    ratatui::restore();
    app_result
}
