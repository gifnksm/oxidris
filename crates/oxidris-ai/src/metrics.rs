use oxidris_engine::{BitBoard, Piece, PieceKind};
use std::fmt;

use crate::BoardAnalysis;

// All evaluation metrics are transformed into a [0.0, 1.0] score,
// where higher is always better.
//
// Normalization ranges are based on practical in-game observations
// (approximately the 95% percentile), not theoretical maxima.
// This preserves resolution and stabilizes GA optimization.

pub const ALL_METRICS: MetricSet<'static, 9> = MetricSet([
    &CoveredHolesMetric,
    &RowTransitionsMetric,
    &ColumnTransitionsMetric,
    &SurfaceRoughnessMetric,
    &MaxHeightMetric,
    &DeepWellRiskMetric,
    &SumOfHeightsMetric,
    &LineClearMetric,
    &IWellRewardMetric,
]);

pub(crate) const ALL_METRICS_COUNT: usize = ALL_METRICS.0.len();

#[derive(Debug, Clone)]
pub struct MetricSet<'a, const N: usize>([&'a dyn DynMetricSource; N]);

impl<const N: usize> MetricSet<'_, N> {
    #[must_use]
    pub fn measure(&self, board: &BitBoard, placement: Piece) -> [f32; N] {
        let analysis = BoardAnalysis::from_board(board, placement);
        self.0.map(|metric| metric.measure(&analysis).normalized)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricSignal {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricValue {
    pub raw: u32,
    pub transformed: f32,
    pub normalized: f32,
}

pub trait MetricSource: fmt::Debug {
    const MAX_VALUE: f32;
    const SIGNAL: MetricSignal;

    #[must_use]
    fn name() -> &'static str;

    #[must_use]
    fn measure_raw(analysis: &BoardAnalysis) -> u32;

    #[must_use]
    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        raw as f32
    }

    #[must_use]
    fn normalize(transformed: f32) -> f32 {
        let norm = (transformed / Self::MAX_VALUE).clamp(0.0, 1.0);
        match Self::SIGNAL {
            MetricSignal::Positive => norm,
            MetricSignal::Negative => 1.0 - norm,
        }
    }

    #[must_use]
    fn measure(analysis: &BoardAnalysis) -> MetricValue {
        let raw = Self::measure_raw(analysis);
        let transformed = Self::transform(raw);
        let normalized = Self::normalize(transformed);
        MetricValue {
            raw,
            transformed,
            normalized,
        }
    }
}

pub trait DynMetricSource: fmt::Debug {
    #[must_use]
    fn name(&self) -> &'static str;
    #[must_use]
    fn max_value(&self) -> f32;
    #[must_use]
    fn signal(&self) -> MetricSignal;
    #[must_use]
    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32;
    #[must_use]
    fn transform(&self, raw: u32) -> f32;
    #[must_use]
    fn normalize(&self, transformed: f32) -> f32;
    #[must_use]
    fn measure(&self, analysis: &BoardAnalysis) -> MetricValue;
}

impl<T> DynMetricSource for T
where
    T: MetricSource,
{
    fn name(&self) -> &'static str {
        T::name()
    }

    fn max_value(&self) -> f32 {
        T::MAX_VALUE
    }

    fn signal(&self) -> MetricSignal {
        T::SIGNAL
    }

    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
        T::measure_raw(analysis)
    }

    fn transform(&self, raw: u32) -> f32 {
        T::transform(raw)
    }

    fn normalize(&self, transformed: f32) -> f32 {
        T::normalize(transformed)
    }

    fn measure(&self, analysis: &BoardAnalysis) -> MetricValue {
        T::measure(analysis)
    }
}

#[derive(Debug)]
pub struct CoveredHolesMetric;

// Covered holes are empty cells with at least one block above them.
// They are one of the strongest losing factors.
//
// Typical ranges (raw hole count):
//   0–3   : very clean board
//   4–7   : dangerous, recovery becomes difficult
//   10+   : near-losing position
//
// A power transform (holes^1.5) emphasizes early hole creation.
// The normalization max (~60) corresponds to ~15 practical holes.
impl MetricSource for CoveredHolesMetric {
    const MAX_VALUE: f32 = 60.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Covered Holes"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        core::iter::zip(analysis.column_heights, analysis.column_occupied_cells)
            .map(|(h, occ)| u32::from(h - occ))
            .sum()
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        (raw as f32).powf(1.5)
    }
}

#[derive(Debug)]
pub struct RowTransitionsMetric;

// Row transitions measure how fragmented each row is by counting
// horizontal changes between occupied and empty cells.
//
// Only transitions *within* the playable area are counted.
// Board walls are intentionally ignored to avoid artificial
// incentives for stacking against the edges.
//
// This metric penalizes:
//   - fragmented horizontal structures
//   - narrow gaps and broken surfaces
//   - layouts that are difficult to clear efficiently
//
// Typical ranges (10x20 board, wall-ignored):
//   20–40   : very clean and flat structure
//   60–90   : normal mid-game roughness
//   120+    : highly fragmented, unstable board
//
// This is a negative metric; lower transition counts are better.
impl MetricSource for RowTransitionsMetric {
    const MAX_VALUE: f32 = 120.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Row Transitions"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        let mut transitions = 0;
        for row in analysis.board.playable_rows() {
            let mut cells = row.iter_playable_cells();
            let mut prev_occupied = cells.next().unwrap(); // left wall
            for occupied in cells {
                if occupied != prev_occupied {
                    transitions += 1;
                }
                prev_occupied = occupied;
            }
        }
        transitions
    }
}

#[derive(Debug)]
pub struct ColumnTransitionsMetric;

// Column transitions count vertical occupancy changes per column,
// treating covered holes as empty cells.
//
// This metric captures vertical fragmentation that is not always
// visible from row transitions alone.
//
// Typical ranges (10x20 board):
//   20–40   : clean vertical structure
//   60–100  : normal mid-game fragmentation
//   120+    : severe vertical instability
//
// Covered holes are intentionally treated as empty here,
// since they are already penalized by a dedicated metric.
impl MetricSource for ColumnTransitionsMetric {
    const MAX_VALUE: f32 = 120.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Column Transitions"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        let mut transitions = 0;
        for x in BitBoard::PLAYABLE_X_RANGE {
            let mut prev_occupied = analysis.board.playable_row(0).is_cell_occupied(x); // top cell
            for y in 1..BitBoard::PLAYABLE_HEIGHT {
                let occupied = analysis.board.playable_row(y).is_cell_occupied(x);
                if occupied != prev_occupied {
                    transitions += 1;
                }
                prev_occupied = occupied;
            }
            if !prev_occupied {
                transitions += 1; // bottom wall
            }
        }
        transitions
    }
}

#[derive(Debug)]
pub struct SurfaceRoughnessMetric;

// Surface roughness measures local curvature of the board surface
// using second-order height differences.
//
// Unlike row transitions, this metric remains sensitive
// when the overall stack is low.
//
// Typical ranges:
//   0–5    : flat or well-shaped surface
//   10–20  : normal mid-game roughness
//   30+    : chaotic surface with high hole risk
//
// This metric complements row transitions rather than replacing it.
impl MetricSource for SurfaceRoughnessMetric {
    const MAX_VALUE: f32 = 40.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Surface Roughness"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        analysis
            .column_heights
            .windows(3)
            .map(|w| {
                let left = i32::from(w[0]);
                let mid = i32::from(w[1]);
                let right = i32::from(w[2]);
                ((right - mid) - (mid - left)).unsigned_abs()
            })
            .sum()
    }
}

#[derive(Debug)]
pub struct MaxHeightMetric;

// Max height represents imminent top-out danger.
// Heights below the safe threshold are intentionally ignored,
// as moderate stacking is often necessary for line clears
// and I-well construction.
// A sharp exponential penalty is applied only near the ceiling
// to model the irreversible nature of top-out.
impl MetricSource for MaxHeightMetric {
    const MAX_VALUE: f32 = 1.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Max Height"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        let max_height = *analysis.column_heights.iter().max().unwrap();
        u32::from(max_height)
    }

    fn transform(raw: u32) -> f32 {
        const SAFE_THRESHOLD: f32 = 0.7;
        #[expect(clippy::cast_precision_loss)]
        let h = (raw as f32) / (BitBoard::PLAYABLE_HEIGHT as f32);
        if h <= SAFE_THRESHOLD {
            0.0
        } else {
            (h - SAFE_THRESHOLD) / (1.0 - SAFE_THRESHOLD)
        }
    }

    fn normalize(norm: f32) -> f32 {
        (-4.0 * norm).exp()
    }
}

#[derive(Debug)]
pub struct DeepWellRiskMetric;

// Deep wells detect excessively deep vertical gaps (width = 1).
// Only wells deeper than 2 are considered dangerous.
//
// raw = sum of (well_depth^2) for dangerous wells
// This aggressively penalizes over-committed vertical structures.
//
// Typical interpretation (10x20 board):
//   raw ≈ 0      : no dangerous wells (safe or controlled I-wells)
//   raw ≈ 10–20  : risky but potentially recoverable
//   raw ≥ 50     : highly unstable, near-fatal structure
//
// This metric is NOT a positive reward.
// It acts purely as a safety penalty using exponential decay,
// while preserving freedom to build shallow I-wells.
impl MetricSource for DeepWellRiskMetric {
    const MAX_VALUE: f32 = 50.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Deep Well Risk"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        analysis
            .column_well_depths
            .iter()
            .filter(|depth| **depth > 2)
            .map(|depth| {
                let depth = u32::from(*depth - 2);
                depth * depth
            })
            .sum()
    }

    fn normalize(norm: f32) -> f32 {
        (-norm).exp()
    }
}

#[derive(Debug)]
pub struct SumOfHeightsMetric;

// Sum of column heights represents overall board pressure.
// It correlates with reduced mobility and imminent top-out.
//
// Typical ranges (sum of heights, 10x20 board):
//   40–60   : early game, very safe
//   80–120  : mid game, manageable
//   140+    : near top-out, highly dangerous
//
// max = 160 is chosen as a "95% practical limit",
// not the theoretical maximum.
impl MetricSource for SumOfHeightsMetric {
    const MAX_VALUE: f32 = 160.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Sum of Heights"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        analysis.column_heights.iter().map(|&h| u32::from(h)).sum()
    }
}

#[derive(Debug)]
pub struct IWellRewardMetric;

impl MetricSource for IWellRewardMetric {
    const MAX_VALUE: f32 = 1.0;
    const SIGNAL: MetricSignal = MetricSignal::Positive;

    fn name() -> &'static str {
        "I-Well Reward"
    }

    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        let mut best_reward = 0.0;
        let mut best_depth = 0.0;
        for (i, depth) in analysis.column_well_depths.into_iter().enumerate() {
            if depth < 1 {
                continue;
            }
            let depth = f32::from(depth);
            let dist_to_edge = usize::min(i, BitBoard::PLAYABLE_WIDTH - 1 - i);
            #[expect(clippy::cast_precision_loss)]
            let dist = (dist_to_edge as f32) / (BitBoard::PLAYABLE_WIDTH as f32 / 2.0);

            let peak = 4.5;
            let sigma = 2.0;
            let depth_score = (-(depth - peak).powi(2) / (2.0 * sigma * sigma)).exp();

            let edge_bonus = (-dist).exp();

            let reward = depth_score * edge_bonus;
            if reward > best_reward {
                best_depth = depth;
                best_reward = reward;
            }
        }
        if best_depth >= 4.0 && analysis.placement.kind() == PieceKind::I {
            return 0;
        }
        (best_reward * 1000.0) as u32
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        (raw as f32) / 1000.0
    }
}

#[derive(Debug)]
pub struct LineClearMetric;

// Lines cleared represent forward progress and efficiency.
// Weights strongly favor tetrises (4-line clears).
impl MetricSource for LineClearMetric {
    const MAX_VALUE: f32 = 6.0;
    const SIGNAL: MetricSignal = MetricSignal::Positive;

    fn name() -> &'static str {
        "Lines Cleared"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        u32::try_from(analysis.cleared_lines).unwrap()
    }

    fn transform(raw: u32) -> f32 {
        const WEIGHT: [f32; 5] = [0.0, 0.0, 1.0, 2.0, 6.0];
        WEIGHT[usize::try_from(raw).unwrap()]
    }
}
