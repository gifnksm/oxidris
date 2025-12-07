use std::io;

use clap::{Parser, ValueEnum};

mod ai;
mod core;
mod engine;
mod modes;
mod ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// What mode to run the program in
    #[arg(value_enum, default_value_t = Mode::Normal)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// Run normal play
    Normal,
    /// Run auto play
    Auto,
    /// Learning with genetic algorithm
    Learning,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    match cli.mode {
        Mode::Normal => modes::normal()?,
        Mode::Auto => modes::auto()?,
        Mode::Learning => ai::genetic::learning(),
    }
    Ok(())
}
