use std::path::PathBuf;

use crate::{
    command::play::screens::{AutoPlayScreen, ManualPlayScreen},
    tui::{ScreenStack, Tui},
    util,
};

mod screens;

const TICK_RATE: f64 = 60.0;

#[derive(Default, Debug, Clone, clap::Args)]
struct RecordingArg {
    /// Save the game recording to a file when the session ends
    #[clap(long)]
    save_recording: bool,
    /// Directory to save recording files
    #[clap(long, default_value = "./data/recordings/")]
    record_dir: PathBuf,
    /// Maximum number of turns to keep in memory (oldest are discarded)
    #[clap(long, default_value_t = 10000)]
    history_size: usize,
}

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct ManualPlayArg {
    #[clap(flatten)]
    recording: RecordingArg,
}

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct AutoPlayArg {
    /// Path to the model file (JSON format)
    model_path: PathBuf,
    /// Run in turbo mode
    #[clap(long, default_value_t = false)]
    turbo: bool,
    #[clap(flatten)]
    recording: RecordingArg,
}

pub(crate) fn run_manual(arg: &ManualPlayArg) -> anyhow::Result<()> {
    let ManualPlayArg {
        recording:
            RecordingArg {
                save_recording,
                record_dir,
                history_size,
            },
    } = arg;

    let mut session_history = None;

    let mut app = ScreenStack::new(Box::new(ManualPlayScreen::new(
        TICK_RATE,
        *history_size,
        &mut session_history,
    )));
    Tui::new().run(&mut app)?;
    drop(app);

    if *save_recording {
        session_history.as_mut().unwrap().save(record_dir)?;
    }

    Ok(())
}

pub(crate) fn run_auto(arg: &AutoPlayArg) -> anyhow::Result<()> {
    let AutoPlayArg {
        model_path,
        turbo,
        recording:
            RecordingArg {
                save_recording,
                record_dir,
                history_size,
            },
    } = arg;

    let mut session_history = None;

    let model = util::read_ai_model_file(model_path)?;
    let mut app = ScreenStack::new(Box::new(AutoPlayScreen::new(
        TICK_RATE,
        &model,
        *history_size,
        *turbo,
        &mut session_history,
    )?));
    Tui::new().run(&mut app)?;
    drop(app);

    if *save_recording {
        session_history.as_mut().unwrap().save(record_dir)?;
    }

    Ok(())
}
