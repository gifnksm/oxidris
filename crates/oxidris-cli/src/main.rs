mod command;
mod schema;
mod ui;
mod util;

fn main() -> anyhow::Result<()> {
    command::run()
}
