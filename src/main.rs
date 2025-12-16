use std::io;

use clap::{Parser, Subcommand, ValueEnum};

mod ai;
mod core;
mod engine;
mod modes;
mod ui;

#[derive(Debug, Clone, Copy, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// What mode to run the program in
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Debug, Copy, Clone, Subcommand)]
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
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum AiType {
    #[default]
    Aggro,
    Defensive,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.mode.unwrap_or(Mode::Normal) {
        Mode::Normal => modes::normal()?,
        Mode::Auto { ai } => modes::auto(ai)?,
        Mode::Learning { ai } => ai::genetic::learning(ai),
    }
    Ok(())
}
