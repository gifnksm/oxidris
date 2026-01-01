use std::{collections::BTreeMap, fs::File, io::BufReader, iter, ops::Range, path::Path};

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use oxidris_ai::{
    board_feature::{ALL_BOARD_FEATURES, BoardFeatureValue},
    weights::WeightSet,
};
use oxidris_engine::{BitBoard, Piece};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoardCollection {
    pub boards: Vec<BoardAndPlacement>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoardAndPlacement {
    pub board: BitBoard,
    pub placement: Piece,
}

#[derive(Debug, Clone)]
pub struct BoardFeatures {
    pub board: BoardAndPlacement,
    pub features: [BoardFeatureValue; ALL_BOARD_FEATURES.len()],
}

#[derive(Debug, Clone)]
pub struct FeatureStatistics {
    pub raw: ValueStats,
    pub transformed: ValueStats,
    pub normalized: ValueStats,
}

#[derive(Debug, Clone)]
pub struct ValueStats {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub median: f32,
    pub std_dev: f32,
    pub percentiles: Vec<(f32, f32)>,
    pub histogram: Histogram,
}

#[derive(Debug, Clone)]
pub struct Histogram {
    pub bins: Vec<HistogramBin>,
}

#[derive(Debug, Clone)]
pub struct HistogramBin {
    pub range: Range<f32>,
    pub count: u64,
}

impl ValueStats {
    pub fn get_percentile(&self, percentile: f32) -> Option<f32> {
        self.percentiles.iter().find_map(|(p, value)| {
            if (*p - percentile).abs() < f32::EPSILON {
                Some(*value)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Model {
    pub name: String,
    pub trained_at: DateTime<Utc>,
    pub final_fitness: f32,
    pub placement_weights: BTreeMap<String, f32>,
}

impl Model {
    pub(crate) fn to_feature_weights(&self) -> WeightSet<{ ALL_BOARD_FEATURES.len() }> {
        let mut weights = [0.0; ALL_BOARD_FEATURES.len()];
        for (feature, slot) in iter::zip(ALL_BOARD_FEATURES.as_array(), &mut weights) {
            if let Some(weight) = self.placement_weights.get(feature.id()) {
                *slot = *weight;
            }
        }
        WeightSet::from_array(weights)
    }
}

pub fn load_board<P>(path: P) -> anyhow::Result<Vec<BoardAndPlacement>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;

    let reader = BufReader::new(file);
    let boards: BoardCollection = serde_json::from_reader(reader)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    if boards.boards.is_empty() {
        bail!("{} is empty", path.display());
    }

    Ok(boards.boards)
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
