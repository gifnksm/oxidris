mod command;
mod record;
mod schema;
mod util;
mod view;

fn main() -> anyhow::Result<()> {
    command::run()
}
