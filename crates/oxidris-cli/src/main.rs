mod command;
mod record;
mod schema;
mod tui;
mod util;
mod view;

const DEFAULT_FRAME_RATE: f64 = 60.0;

fn main() -> anyhow::Result<()> {
    command::run()
}
