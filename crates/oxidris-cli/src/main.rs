mod command;
mod record;
mod schema;
mod tui;
mod util;
mod view;

fn main() -> anyhow::Result<()> {
    command::run()
}
