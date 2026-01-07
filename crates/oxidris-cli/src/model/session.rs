use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, bail};
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

impl SessionCollection {
    pub fn open<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let file =
            File::open(path).with_context(|| format!("failed to open {}", path.display()))?;

        let reader = BufReader::new(file);
        let boards: SessionCollection = serde_json::from_reader(reader)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        if boards.sessions.is_empty() {
            bail!("{} is empty", path.display());
        }

        Ok(boards)
    }
}
