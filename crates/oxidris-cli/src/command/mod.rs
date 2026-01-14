use clap::{Parser, Subcommand};

use crate::command::play::ManualPlayArg;

use self::{
    analyze_board_features::AnalyzeBoardFeaturesArg, analyze_censoring::AnalyzeCensoringArg,
    generate_boards::GenerateBoardsArg, train_ai::TrainAiArg,
};

mod analyze_board_features;
mod analyze_censoring;
mod generate_boards;
mod play;
mod train_ai;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct CommandArgs {
    /// What mode to run the program in
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Debug, Clone, Subcommand)]
enum Mode {
    #[command(name = "play")]
    ManualPlay(#[clap(flatten)] play::ManualPlayArg),
    #[command(name = "auto-play")]
    AutoPlay(#[clap(flatten)] play::AutoPlayArg),
    /// Train AI using genetic algorithm
    TrainAi(#[clap(flatten)] TrainAiArg),
    /// Generate boards for training data
    GenerateBoards(#[clap(flatten)] GenerateBoardsArg),
    /// Analyze board features with TUI
    AnalyzeBoardFeatures(#[clap(flatten)] AnalyzeBoardFeaturesArg),
    /// Analyze censoring in board data
    AnalyzeCensoring(#[clap(flatten)] AnalyzeCensoringArg),
}

pub fn run() -> anyhow::Result<()> {
    let args = CommandArgs::parse();
    match args
        .mode
        .unwrap_or(Mode::ManualPlay(ManualPlayArg::default()))
    {
        Mode::ManualPlay(arg) => play::run_manual(&arg)?,
        Mode::AutoPlay(arg) => play::run_auto(&arg)?,
        Mode::TrainAi(arg) => train_ai::run(&arg)?,
        Mode::GenerateBoards(arg) => generate_boards::run(&arg)?,
        Mode::AnalyzeBoardFeatures(arg) => analyze_board_features::run(&arg)?,
        Mode::AnalyzeCensoring(arg) => analyze_censoring::run(&arg)?,
    }
    Ok(())
}
