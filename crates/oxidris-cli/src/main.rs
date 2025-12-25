use std::path::PathBuf;

use clap::{Parser, Subcommand};
use oxidris_ai::AiType;

mod data;
mod generate_boards;
mod play;
mod tune_metrics;
mod ui;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// What mode to run the program in
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Debug, Clone, Subcommand)]
enum Mode {
    /// Run normal play
    Normal,
    /// Run auto play
    Auto {
        #[arg(long, default_value = "aggro")]
        ai: AiType,
    },
    /// Learning with genetic algorithm
    Learning {
        #[arg(long, default_value = "aggro")]
        ai: AiType,
    },
    /// Generate boards for training data
    GenerateBoards,
    /// Tune metrics weights
    TuneMetrics { boards_file: PathBuf },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.mode.unwrap_or(Mode::Normal) {
        Mode::Normal => play::normal()?,
        Mode::Auto { ai } => play::auto(ai)?,
        Mode::Learning { ai } => oxidris_ai::genetic::learning(ai),
        Mode::GenerateBoards => generate_boards::run(),
        Mode::TuneMetrics { boards_file } => tune_metrics::run(&boards_file)?,
    }
    Ok(())
}
