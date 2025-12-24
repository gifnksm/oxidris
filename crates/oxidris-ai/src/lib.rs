pub use self::{
    board_analysis::*, metrics::*, placement_evaluator::*, turn_evaluator::*, weights::*,
};

mod board_analysis;
pub mod genetic;
mod metrics;
mod placement_evaluator;
mod turn_evaluator;
mod weights;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, derive_more::FromStr)]
pub enum AiType {
    #[default]
    Aggro,
    Defensive,
}
