use std::path::Path;

mod analysis;
mod data;
mod index;
mod ui;

pub fn run(boards_file: &Path) -> anyhow::Result<()> {
    eprintln!("Loading boards from {}...", boards_file.display());
    let boards = data::load_board(boards_file)?;
    eprintln!("Loaded {} boards", boards.len());

    eprintln!("Computing metrics for all boards...");
    let boards_metrics = analysis::compute_all_metrics(&boards);
    eprintln!("Metrics computed");

    eprintln!("Computing statistics");
    let statistics = analysis::coimpute_statistics(&boards_metrics);
    eprintln!("Statistics computed");

    eprintln!("Building board index...");
    let board_index = index::BoardIndex::new(&boards_metrics);
    eprintln!("Board index built");

    ui::run_tui(boards_metrics, statistics, board_index)?;

    Ok(())
}
