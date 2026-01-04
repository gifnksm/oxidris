//! Core data structures for the Tetris game engine.
//!
//! This module provides fundamental types and representations used throughout the engine:
//!
//! - [`Piece`] - Tetromino pieces with position, rotation, and shape
//! - [`BitBoard`] - Efficient bitboard representation for collision detection
//! - [`BlockBoard`] - Cell-by-cell board representation for rendering and analysis
//!
//! # Board Dimensions
//!
//! The game uses standard Tetris dimensions with additional sentinel margins for
//! efficient boundary checking:
//!
//! - **Playable area**: 10 columns × 20 rows
//! - **Total dimensions**: 14 columns × 24 rows (includes 2-cell margins on all sides)
//!
//! The sentinel margins allow collision detection without explicit boundary checks,
//! improving performance during AI search.
//!
//! # Coordinate System
//!
//! - Origin (0, 0) is at the top-left of the playable area
//! - X increases rightward (columns)
//! - Y increases downward (rows)
//! - Piece coordinates are relative to their anchor point

pub use self::{bit_board::*, block_board::*, piece::*};

pub(crate) mod bit_board;
pub(crate) mod block_board;
pub(crate) mod piece;

/// Width of the playable game area in columns.
const PLAYABLE_WIDTH: usize = 10;

/// Height of the playable game area in rows.
const PLAYABLE_HEIGHT: usize = 20;

/// Total width including sentinel margins.
const TOTAL_WIDTH: usize = PLAYABLE_WIDTH + (SENTINEL_MARGIN_LEFT + SENTINEL_MARGIN_RIGHT);

/// Total height including sentinel margins.
const TOTAL_HEIGHT: usize = PLAYABLE_HEIGHT + (SENTINEL_MARGIN_TOP + SENTINEL_MARGIN_BOTTOM);

/// Sentinel margin above playable area (for piece spawning).
const SENTINEL_MARGIN_TOP: usize = 2;

/// Sentinel margin below playable area (for boundary detection).
const SENTINEL_MARGIN_BOTTOM: usize = 2;

/// Sentinel margin left of playable area (for boundary detection).
const SENTINEL_MARGIN_LEFT: usize = 2;

/// Sentinel margin right of playable area (for boundary detection).
const SENTINEL_MARGIN_RIGHT: usize = 2;
