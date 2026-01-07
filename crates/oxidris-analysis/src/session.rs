//! Session data structures for training data collection
//!
//! This module provides data structures for representing gameplay sessions
//! and board states captured during training data generation.
//!
//! # Overview
//!
//! Training data consists of game sessions played by various evaluators,
//! with board states captured at each turn. This data is used for:
//!
//! - Computing feature statistics and normalization parameters
//! - Training AI models via genetic algorithms
//! - Analyzing feature distributions and survival patterns
//! - Validating evaluator performance
//!
//! # Data Structure
//!
//! ```text
//! SessionCollection
//! ├─ metadata (total_boards, max_turns)
//! └─ sessions: Vec<SessionData>
//!     ├─ evaluator name
//!     ├─ survival info (turns, is_game_over)
//!     └─ boards: Vec<BoardAndPlacement>
//!         ├─ turn number
//!         ├─ board state (before placement)
//!         └─ piece placed
//! ```
//!
//! # Right-Censored Data
//!
//! Sessions may end in two ways:
//!
//! - **Game Over** (`is_game_over = true`): Board reached terminal state
//! - **Max Turns** (`is_game_over = false`): Survived to turn limit (censored)
//!
//! Censored data must be handled properly in survival analysis using
//! Kaplan-Meier estimation (see [`survival`](crate::survival) module).
//!
//! # Serialization
//!
//! All types implement `serde` traits for JSON serialization:
//!
//! ```json
//! {
//!   "total_boards": 150000,
//!   "max_turns": 500,
//!   "sessions": [
//!     {
//!       "placement_evaluator": "random",
//!       "survived_turns": 45,
//!       "is_game_over": true,
//!       "boards": [...]
//!     }
//!   ]
//! }
//! ```
//!
//! # Examples
//!
//! ## Working with Session Data
//!
//! ```no_run
//! use oxidris_analysis::session::{SessionCollection, SessionData};
//!
//! // In practice, load from JSON file using serde_json
//! let collection = SessionCollection {
//!     total_boards: 150000,
//!     max_turns: 500,
//!     sessions: vec![], // Load from file
//! };
//!
//! println!("Loaded {} sessions", collection.sessions.len());
//! println!("Total boards: {}", collection.total_boards);
//! println!("Max turns: {}", collection.max_turns);
//!
//! // Count censored vs complete sessions
//! let censored = collection
//!     .sessions
//!     .iter()
//!     .filter(|s| !s.is_game_over)
//!     .count();
//! println!(
//!     "Censored sessions: {}/{}",
//!     censored,
//!     collection.sessions.len()
//! );
//! ```
//!
//! ## Iterating Over Boards
//!
//! ```no_run
//! use oxidris_analysis::session::{SessionCollection, SessionData};
//!
//! // Load collection from file (using serde_json in practice)
//! let collection = SessionCollection {
//!     total_boards: 150000,
//!     max_turns: 500,
//!     sessions: vec![], // Load from file
//! };
//!
//! // Process all boards across all sessions
//! for session in &collection.sessions {
//!     for board in &session.boards {
//!         println!("Turn {}: {:?}", board.turn, board.placement);
//!         // Analyze board.before_placement
//!     }
//! }
//! ```

use oxidris_engine::{BitBoard, Piece};
use serde::{Deserialize, Serialize};

/// Collection of game sessions with board states captured during training data generation.
///
/// This structure represents the output of the `generate-boards` command, containing
/// multiple game sessions played by various placement evaluators.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionCollection {
    /// Total number of board states captured across all sessions
    pub total_boards: usize,
    /// Maximum number of turns allowed per session
    pub max_turns: usize,
    /// List of individual game sessions
    pub sessions: Vec<SessionData>,
}

/// Data for a single game session, including all captured board states.
///
/// Each session represents one complete game played by a specific placement evaluator,
/// either ending in game over or reaching the maximum turn limit.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionData {
    /// Name of the placement evaluator used in this session
    pub placement_evaluator: String,
    /// Number of turns survived in this session
    pub survived_turns: usize,
    /// Whether the session ended due to game over (true) or reaching max turns (false)
    pub is_game_over: bool,
    /// Captured board states and the pieces placed on them during this session
    pub boards: Vec<BoardAndPlacement>,
}

/// A captured board state with the piece that was placed on it.
///
/// Represents a single training data point, containing the board state before
/// a piece placement and the piece that was actually placed by the evaluator.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoardAndPlacement {
    /// Turn number when this board state was captured (0-indexed)
    pub turn: usize,
    /// Board state before the piece placement
    pub before_placement: BitBoard,
    /// Piece that was placed on this board
    pub placement: Piece,
}
