use std::path::PathBuf;

use oxidris_evaluator::board_feature;

use crate::{
    analysis::{BoardFeatureStatistics, BoardIndex, BoardSample},
    model::session::SessionCollection,
};

mod ui;

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct AnalyzeBoardFeaturesArg {
    /// Boards data file path
    boards_file: PathBuf,
}

pub fn run(arg: &AnalyzeBoardFeaturesArg) -> anyhow::Result<()> {
    let AnalyzeBoardFeaturesArg { boards_file } = arg;

    let features = board_feature::all_board_features();

    eprintln!("Loading boards from {}...", boards_file.display());
    let sessions = SessionCollection::open(boards_file)?.sessions;
    eprintln!("Loaded {} sessions", sessions.len());

    eprintln!("Computing featuress for all boards...");
    let board_samples = BoardSample::from_sessions(&features, &sessions);
    eprintln!("Features computed");

    eprintln!("Computing statistics");
    let statistics = BoardFeatureStatistics::from_samples(&features, &board_samples);
    eprintln!("Statistics computed");

    eprintln!("Building board index...");
    let board_index = BoardIndex::from_samples(&features, &board_samples);
    eprintln!("Board index built");

    ui::run_tui(features, board_samples, statistics, board_index)?;

    Ok(())
}
