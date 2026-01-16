use std::path::PathBuf;

use crate::{command::replay::app::ReplayApp, schema::record::RecordedSession, tui::Tui, util};

mod app;
mod screens;

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

    let mut app = ReplayApp::new(recording_file.clone(), session);
    Tui::new().run(&mut app)?;

    Ok(())
}
