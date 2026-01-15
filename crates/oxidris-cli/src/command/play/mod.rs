use std::path::PathBuf;

use oxidris_evaluator::{
    placement_evaluator::FeatureBasedPlacementEvaluator, turn_evaluator::TurnEvaluator,
};

use crate::{command::play::app::App, util};

mod app;
mod screens;

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct ManualPlayArg {
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
pub(crate) struct AutoPlayArg {
    /// Path to the model file (JSON format)
    model_path: PathBuf,
    /// Run in turbo mode
    #[clap(long, default_value_t = false)]
    turbo: bool,
}

pub(crate) fn run_manual(arg: &ManualPlayArg) -> anyhow::Result<()> {
    let ManualPlayArg {
        save_recording,
        record_dir,
        history_size,
    } = arg;

    let mut app = App::manual(*history_size);

    ratatui::run(|terminal| app.run(terminal))?;

    if *save_recording {
        let history = app.into_history().unwrap();
        history.save(record_dir)?;
    }

    Ok(())
}

pub(crate) fn run_auto(arg: &AutoPlayArg) -> anyhow::Result<()> {
    let AutoPlayArg { model_path, turbo } = arg;

    let model = util::read_ai_model_file(model_path)?;
    let (features, weights) = model.to_feature_weights()?;
    let placement_evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
    let turn_evaluator = TurnEvaluator::new(Box::new(placement_evaluator));

    let mut terminal = ratatui::init();
    let app_result = App::auto(turn_evaluator, *turbo).run(&mut terminal);
    ratatui::restore();
    app_result
}
