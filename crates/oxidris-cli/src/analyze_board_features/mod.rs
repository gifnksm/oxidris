use std::path::PathBuf;

use crate::{analysis, data};

mod index;
mod ui;

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct AnalyzeBoardFeaturesArg {
    /// Boards data file path
    boards_file: PathBuf,
}

pub fn run(arg: &AnalyzeBoardFeaturesArg) -> anyhow::Result<()> {
    let AnalyzeBoardFeaturesArg { boards_file } = arg;

    eprintln!("Loading boards from {}...", boards_file.display());
    let boards = data::load_board(boards_file)?;
    eprintln!("Loaded {} boards", boards.len());

    eprintln!("Computing featuress for all boards...");
    let board_samples = analysis::extract_all_board_features(&boards);
    eprintln!("Features computed");

    eprintln!("Computing statistics");
    let statistics = analysis::coimpute_statistics(&board_samples);
    eprintln!("Statistics computed");

    eprintln!("Building board index...");
    let board_index = index::BoardIndex::new(&board_samples);
    eprintln!("Board index built");

    ui::run_tui(board_samples, statistics, board_index)?;

    Ok(())
}
