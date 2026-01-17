//! Tetris game engine for AI training, evaluation, and human play.
//!
//! This crate provides a Tetris-like game engine that supports both human players
//! and AI agents. It implements core Tetris mechanics with some simplifications that
//! make it suitable for training, statistical analysis, and gameplay.
//!
//! # Architecture
//!
//! The crate is organized into two main modules:
//!
//! - [`core`] - Core data structures (pieces, board representations)
//! - [`engine`] - Game logic (field state, session management, piece generation)
//!
//! # Important Design Decisions
//!
//! ## Simplified Rotation System
//!
//! This engine uses a **simplified rotation system**, not full Super Rotation System (SRS):
//!
//! - Basic rotation with simple 4-direction wall kicks (up, down, left, right)
//! - No official SRS kick tables or piece-specific patterns
//! - No T-spin detection or spin scoring
//!
//! **Rationale**: The simplified system is consistent and deterministic, reducing the
//! search space for AI placement evaluation while still allowing most reasonable placements.
//! This makes it well-suited for training and analysis, though strategies learned here
//! may not transfer directly to standard Tetris.
//!
//! ## Standard Features
//!
//! The following are implemented according to modern Tetris guidelines:
//!
//! - ✅ 7-bag piece generation (even distribution, no long droughts)
//! - ✅ Standard hold system
//! - ✅ 10×20 board dimensions
//! - ✅ Standard piece shapes and rotations
//!
//! # Usage
//!
//! The engine supports both programmatic control (for AI agents) and human play.
//! Typical usage involves creating a [`GameField`] or [`GameSession`] and executing
//! moves:
//!
//! ```
//! use oxidris_engine::GameField;
//!
//! let mut field = GameField::new();
//!
//! // Manipulate the falling piece
//! if let Some(piece) = field.falling_piece().left() {
//!     field.set_falling_piece(piece).ok();
//! }
//!
//! if let Some(piece) = field
//!     .falling_piece()
//!     .rotated_right()
//!     .super_rotated_right(field.board())
//! {
//!     field.set_falling_piece(piece).ok();
//! }
//!
//! // Complete the placement
//! let (lines_cleared, result) = field.complete_piece_drop();
//! ```
//!
//! For high-level gameplay with automatic gravity and statistics, use [`GameSession`]:
//!
//! ```
//! use oxidris_engine::GameSession;
//!
//! let mut session = GameSession::new(60.0); // 60 FPS
//!
//! // Move and rotate
//! session.try_move_left().ok();
//! session.try_rotate_right().ok();
//!
//! // Hard drop
//! session.hard_drop_and_complete();
//! ```
//!
//! # Implementation Details
//!
//! For detailed information about mechanics, simplifications, and implications for both
//! AI development and gameplay, see the [Engine Implementation documentation](https://docs/architecture/engine/README.md).
//!
//! Key topics covered there:
//!
//! - Rotation system details and limitations
//! - What features are simplified vs. standard
//! - Performance optimizations (bit-board representation)
//! - Implications for AI training, evaluation, and human play

pub use self::{core::*, engine::*};

pub mod core;
pub mod engine;

/// Error indicating a piece collision occurred when setting the falling piece.
///
/// This error is returned when attempting to place a piece in a position that
/// would overlap with blocks already on the board or exceed board boundaries.
#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display("piece colliding when setting falling piece")]
pub struct PieceCollisionError;

/// Error that can occur when attempting to hold a piece.
///
/// The hold operation can fail for two reasons:
///
/// 1. The piece being swapped in would collide with the board
/// 2. Hold has already been used in the current turn (can only hold once per piece)
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum HoldError {
    /// The piece being swapped in collides with blocks on the board.
    #[display("piece colliding when holding piece")]
    PieceCollision(PieceCollisionError),
    /// Hold has already been used in the current turn.
    ///
    /// The hold action is only allowed once per falling piece.
    #[display("hold already used in this turn")]
    HoldAlreadyUsed,
}

/// Error that can occur when completing a piece drop.
///
/// After locking a piece and clearing lines, the next piece must be spawned.
/// If there is no space for the new piece, the game is over (top-out condition).
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum CompletePieceDropError {
    /// No space to spawn the next piece after completing the drop.
    ///
    /// This represents the standard Tetris game-over condition (top-out).
    #[display("no space to spawn new piece after completing drop")]
    NewPieceCollision,
}
