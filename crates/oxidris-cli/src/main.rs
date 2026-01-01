use clap::{Parser, Subcommand};

use self::{
    analyze_board_features::AnalyzeBoardFeaturesArg,
    generate_board_feature_stats::GenerateBoardFeatureStatsArg,
    generate_boards::GenerateBoardsArg,
    play::{AutoPlayArg, ManualPlayArg},
    train_ai::TrainAiArg,
};

mod analysis;
mod analyze_board_features;
mod data;
mod generate_board_feature_stats;
mod generate_boards;
mod play;
mod train_ai;
mod ui;
mod util;

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
    /// Analyze board features with TUI
    AnalyzeBoardFeatures(#[clap(flatten)] AnalyzeBoardFeaturesArg),
    /// Generate statistics about board features
    GenerateBoardFeatureStats(#[clap(flatten)] GenerateBoardFeatureStatsArg),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli
        .mode
        .unwrap_or(Mode::ManualPlay(ManualPlayArg::default()))
    {
        Mode::ManualPlay(arg) => play::manual(&arg)?,
        Mode::AutoPlay(arg) => play::auto(&arg)?,
        Mode::TrainAi(arg) => train_ai::run(&arg)?,
        Mode::GenerateBoards(arg) => generate_boards::run(&arg)?,
        Mode::AnalyzeBoardFeatures(arg) => analyze_board_features::run(&arg)?,
        Mode::GenerateBoardFeatureStats(arg) => generate_board_feature_stats::run(&arg)?,
    }
    Ok(())
}
