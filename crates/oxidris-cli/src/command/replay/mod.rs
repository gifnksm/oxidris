use std::path::PathBuf;

use crate::{command::replay::app::App, schema::record::RecordedSession, util};

mod app;

#[derive(Debug, Clone, clap::Args)]
pub struct ReplayArg {
    /// Path to the recording file (JSON format)
    recording_file: PathBuf,
}

pub fn run(arg: &ReplayArg) -> anyhow::Result<()> {
    let ReplayArg { recording_file } = arg;

    eprintln!("Loading recording from {}", recording_file.display());
    let session: RecordedSession = util::read_json_file("recording", recording_file)?;

    eprintln!("Loaded {:?} boards", session.boards.len());

    let mut app = App::new(recording_file.clone(), session);

    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
