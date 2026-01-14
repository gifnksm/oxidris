use std::path::PathBuf;

use oxidris_evaluator::{
    placement_evaluator::FeatureBasedPlacementEvaluator, turn_evaluator::TurnEvaluator,
};

use crate::{command::play::app::App, util};

mod app;
mod screens;

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct ManualPlayArg {}

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct AutoPlayArg {
    /// Path to the model file (JSON format)
    model_path: PathBuf,
}

pub(crate) fn run_manual(arg: &ManualPlayArg) -> anyhow::Result<()> {
    let ManualPlayArg {} = arg;

    let mut terminal = ratatui::init();
    let app_result = App::manual().run(&mut terminal);
    ratatui::restore();
    app_result
}

pub(crate) fn run_auto(arg: &AutoPlayArg) -> anyhow::Result<()> {
    let AutoPlayArg { model_path } = arg;

    let model = util::read_ai_model_file(model_path)?;
    let (features, weights) = model.to_feature_weights()?;
    let placement_evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
    let turn_evaluator = TurnEvaluator::new(Box::new(placement_evaluator));

    let mut terminal = ratatui::init();
    let app_result = App::auto(turn_evaluator).run(&mut terminal);
    ratatui::restore();
    app_result
}
