use std::{collections::BTreeMap, fs::File, io::BufReader, ops::Range, path::Path};

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use oxidris_ai::board_feature::{ALL_BOARD_FEATURES, BoardFeatureValue, DynBoardFeatureSource};
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
pub struct BoardSample {
    pub board: BoardAndPlacement,
    pub feature_vector: Vec<BoardFeatureValue>,
}

#[derive(Debug, Clone)]
pub struct BoardFeatureStatistics {
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
