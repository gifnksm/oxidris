pub use self::{
    board_analysis::*, board_feature::*, placement_evaluator::*, turn_evaluator::*, weights::*,
};

mod board_analysis;
mod board_feature;
pub mod genetic;
mod placement_evaluator;
mod turn_evaluator;
mod weights;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, derive_more::FromStr)]
pub enum AiType {
    #[default]
    Aggro,
    Defensive,
}
