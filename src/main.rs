use clap::{Parser, ValueEnum};

mod ai;
mod block;
mod ga;
mod game;
mod mino;
mod play;
mod terminal;

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
    /// Learning with GeneticAlgorithm
    Learning,
}

fn main() {
    let cli = Cli::parse();
    match cli.mode {
        Mode::Normal => play::normal(),
        Mode::Auto => play::auto(),
        Mode::Learning => ga::learning(),
    }
}
