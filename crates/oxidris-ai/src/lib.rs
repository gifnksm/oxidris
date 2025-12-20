pub use self::{
    turn_evaluator::{TurnEvaluator, TurnPlan},
    weights::WeightSet,
};

pub mod genetic;
mod metrics;
mod turn_evaluator;
mod weights;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, derive_more::FromStr)]
pub enum AiType {
    #[default]
    Aggro,
    Defensive,
}
