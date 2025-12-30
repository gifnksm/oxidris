use clap::{Parser, Subcommand};
use oxidris_ai::AiType;

use self::{
    generate_boards::GenerateBoardsArg,
    play::{AutoPlayArg, ManualPlayArg},
    train_ai::TrainAiArg,
};

mod data;
mod generate_boards;
mod play;
mod train_ai;
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
    #[command(name = "play")]
    ManualPlay(#[clap(flatten)] ManualPlayArg),
    /// Run auto play
    AutoPlay(#[clap(flatten)] AutoPlayArg),
    /// Train AI using genetic algorithm
    TrainAi(#[clap(flatten)] TrainAiArg),
    /// Generate boards for training data
    GenerateBoards(#[clap(flatten)] GenerateBoardsArg),
    /// Tune metrics weights
    TuneMetrics(#[clap(flatten)] tune_metrics::GenerateBoardsArg),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli
        .mode
        .unwrap_or(Mode::ManualPlay(ManualPlayArg::default()))
    {
        Mode::ManualPlay(arg) => play::manual(&arg)?,
        Mode::AutoPlay(arg) => play::auto(&arg)?,
        Mode::TrainAi(arg) => train_ai::run(&arg),
        Mode::GenerateBoards(arg) => generate_boards::run(&arg)?,
        Mode::TuneMetrics(arg) => tune_metrics::run(&arg)?,
    }
    Ok(())
}
