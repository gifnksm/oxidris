pub mod board_analysis;
pub mod board_feature;
pub mod genetic;
pub mod placement_analysis;
pub mod placement_evaluator;
pub mod session_evaluator;
pub mod turn_evaluator;
pub mod weights;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, derive_more::FromStr)]
pub enum AiType {
    #[default]
    Aggro,
    Defensive,
}
