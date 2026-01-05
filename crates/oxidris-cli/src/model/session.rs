use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, bail};
use oxidris_engine::{BitBoard, Piece};
use oxidris_evaluator::board_feature::BoardFeatureValue;
use oxidris_stats::comprehensive::ComprehensiveStats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionCollection {
    pub total_boards: usize,
    pub max_turns: usize,
    pub sessions: Vec<SessionData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionData {
    pub placement_evaluator: String,
    pub survived_turns: usize,
    pub is_game_over: bool,
    pub boards: Vec<BoardAndPlacement>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoardAndPlacement {
    pub turn: usize,
    pub board: BitBoard,
    pub placement: Piece,
}

#[derive(Debug, Clone)]
pub struct BoardSample {
    #[expect(unused, reason = "may be used later")] // TODO
    pub board: BoardAndPlacement,
    pub feature_vector: Vec<BoardFeatureValue>,
}

#[derive(Debug, Clone)]
pub struct BoardFeatureStatistics {
    pub raw: ComprehensiveStats,
    pub transformed: ComprehensiveStats,
    pub normalized: ComprehensiveStats,
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
