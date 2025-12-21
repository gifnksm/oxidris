pub use self::{turn_evaluator::*, weights::*};

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
