mod command;
mod model;
mod ui;
mod util;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayMode {
    Manual,
    Auto,
}

fn main() -> anyhow::Result<()> {
    command::run()
}
