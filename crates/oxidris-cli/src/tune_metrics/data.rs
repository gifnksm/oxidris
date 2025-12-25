use std::{fs::File, io::BufReader, ops::Range, path::Path};

use anyhow::{Context, bail};
use oxidris_ai::{ALL_METRICS, MetricMeasurement};

use crate::data::{BoardAndPlacement, BoardCollection};

#[derive(Debug, Clone)]
pub struct BoardMetrics {
    pub board: BoardAndPlacement,
    pub metrics: [MetricMeasurement; ALL_METRICS.len()],
}

#[derive(Debug, Clone)]
pub struct MetricStatistics {
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
