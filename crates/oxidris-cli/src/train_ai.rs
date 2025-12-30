use oxidris_ai::AiType;

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct TrainAiArg {
    #[arg(long, default_value = "aggro")]
    ai: AiType,
}

pub(crate) fn run(arg: &TrainAiArg) {
    let TrainAiArg { ai } = arg;
    oxidris_ai::genetic::learning(*ai);
}
