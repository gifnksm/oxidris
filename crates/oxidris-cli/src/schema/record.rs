use chrono::{DateTime, Utc};
use oxidris_engine::{BitBoard, GameStats, Piece, PieceSeed};
use serde::{Deserialize, Serialize};

use crate::schema::ai_model::AiModel;

/// Recorded play session with metadata for replay functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedSession {
    /// Timestamp when recording was created (ISO 8601 format)
    pub recorded_at: DateTime<Utc>,
    /// Random seed used for piece generation
    pub seed: PieceSeed,
    /// Player information (manual or AI with model data)
    pub player: PlayerInfo,
    /// Final game statistics at the time of recording
    pub final_stats: GameStats,
    /// Sequence of board states and piece placements during the session
    pub boards: Vec<TurnRecord>,
}

/// A single turn record capturing the board state before piece placement.
///
/// Each record represents one completed piece placement, storing the state
/// immediately before the piece was locked into place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnRecord {
    /// Turn number (0-indexed, increments with each piece placement)
    pub turn: usize,
    /// Board state before the piece was placed
    pub before_placement: BitBoard,
    /// The piece that was placed (includes position and rotation)
    pub placement: Piece,
    /// Whether hold was used during this turn
    pub hold_used: bool,
}

/// Information about the player type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerInfo {
    /// Manual play by human
    Manual,
    /// AI play with full model data
    Auto { model: AiModel },
}
