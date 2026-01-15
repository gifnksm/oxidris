//! Game engine logic and state management.
//!
//! This module provides the high-level game logic that orchestrates the core data
//! structures to implement Tetris gameplay:
//!
//! - [`GameField`] - Single-turn game state (board, falling piece, next pieces, hold)
//! - [`GameSession`] - Multi-turn game session with statistics tracking
//! - [`GameStats`] - Game statistics (lines cleared, score, survival time)
//! - [`PieceBuffer`] - 7-bag piece generation system
//! - [`PieceSeed`] - Seed for deterministic piece generation
//!
//! # Game Flow
//!
//! A typical game progresses as follows:
//!
//! 1. Initialize [`GameField`] with a random seed
//! 2. Player/AI manipulates the falling piece (move, rotate, hold)
//! 3. Complete the placement with hard drop
//! 4. Lines are cleared and new piece spawns
//! 5. Repeat until top-out (piece collision at spawn)
//!
//! For multi-turn sessions with statistics, use [`GameSession`] which wraps
//! [`GameField`] and tracks performance metrics.
//!
//! # Example
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
//! if let Some(piece) = field.falling_piece().super_rotated_right(field.board()) {
//!     field.set_falling_piece(piece).ok();
//! }
//!
//! // Complete the placement
//! let (lines_cleared, result) = field.complete_piece_drop();
//!
//! if result.is_err() {
//!     println!("Game over!");
//! }
//! ```

pub use self::{game_field::*, game_session::*, game_stats::*, piece_buffer::*};

mod game_field;
mod game_session;
mod game_stats;
mod piece_buffer;
