mod command;
mod model;
mod ui;
mod util;

fn main() -> anyhow::Result<()> {
    command::run()
}
