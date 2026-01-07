use std::path::PathBuf;

use oxidris_analysis::{
    index::BoardIndex, sample::BoardSample, statistics::BoardFeatureStatistics,
};

use crate::util;

mod ui;

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct AnalyzeBoardFeaturesArg {
    /// Board data file path
    boards_file: PathBuf,
}

pub fn run(arg: &AnalyzeBoardFeaturesArg) -> anyhow::Result<()> {
    let AnalyzeBoardFeaturesArg { boards_file } = arg;

    eprintln!("Loading boards from {}...", boards_file.display());
    let sessions = util::read_boards_file(boards_file)?.sessions;
    eprintln!("Loaded {} sessions", sessions.len());

    let features = util::build_feature_from_session(&sessions)?;

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
