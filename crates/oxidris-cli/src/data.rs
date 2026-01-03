use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use oxidris_ai::board_feature::{ALL_BOARD_FEATURES, BoardFeatureValue, DynBoardFeatureSource};
use oxidris_engine::{BitBoard, Piece};
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
    pub board: BoardAndPlacement,
    pub feature_vector: Vec<BoardFeatureValue>,
}

#[derive(Debug, Clone)]
pub struct BoardFeatureStatistics {
    pub raw: ComprehensiveStats,
    pub transformed: ComprehensiveStats,
    pub normalized: ComprehensiveStats,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Model {
    pub name: String,
    pub trained_at: DateTime<Utc>,
    pub final_fitness: f32,
    pub placement_weights: BTreeMap<String, f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NormalizationParams {
    pub max_turns: usize,
    pub normalization_method: String,
    pub features: BTreeMap<String, FeatureNormalization>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureNormalization {
    pub mapping: BTreeMap<u32, f64>,
    pub stats: NormalizationStats,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NormalizationStats {
    pub p05_feature_value: u32,
    pub p95_feature_value: u32,
    pub p05_km_median: f64,
    pub p95_km_median: f64,
    pub total_unique_values: usize,
}

impl NormalizationStats {
    /// Calculate the KM median range (difference in survival time)
    pub fn km_range(&self) -> f64 {
        self.p05_km_median - self.p95_km_median
    }
}

impl Model {
    pub(crate) fn to_feature_weights(
        &self,
    ) -> anyhow::Result<(Vec<&'static dyn DynBoardFeatureSource>, Vec<f32>)> {
        self.placement_weights
            .iter()
            .map(|(feature_id, weight)| -> anyhow::Result<(&'static dyn DynBoardFeatureSource, f32)> {
                let feature = ALL_BOARD_FEATURES
                    .iter()
                    .find(|f| f.id() == feature_id)
                    .ok_or_else(|| anyhow::anyhow!("Feature ID {feature_id} in model not found in ALL_BOARD_FEATURES"))?;
                Ok((*feature, *weight))
            })
            .collect()
    }
}

pub fn load_session_collection<P>(path: P) -> anyhow::Result<SessionCollection>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;

    let reader = BufReader::new(file);
    let boards: SessionCollection = serde_json::from_reader(reader)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    if boards.sessions.is_empty() {
        bail!("{} is empty", path.display());
    }

    Ok(boards)
}

pub fn load_model<P>(path: P) -> anyhow::Result<Model>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let file = File::open(path)
        .with_context(|| format!("Failed to open model file: {}", path.display()))?;

    let reader = BufReader::new(file);
    let model: Model = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to read model file: {}", path.display()))?;

    Ok(model)
}
