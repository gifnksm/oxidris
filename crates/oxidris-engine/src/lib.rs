pub use self::{core::*, engine::*};

pub mod core;
pub mod engine;

#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("piece colliding when setting falling piece")]
pub struct PieceCollisionError;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum HoldError {
    #[display("piece colliding when holding piece")]
    PieceCollision(PieceCollisionError),
    #[display("hold already used in this turn")]
    HoldAlreadyUsed,
}
