use oxidris_engine::{BitBoard, Piece, PieceKind};
use std::fmt;

use crate::BoardAnalysis;

// All evaluation metrics are transformed into a [0.0, 1.0] score,
// where higher is always better.
//
// Normalization ranges are based on practical in-game observations
// (approximately the 95% percentile), not theoretical maxima.
// This preserves resolution and stabilizes GA optimization.

#[derive(Clone)]
pub(crate) struct MetricValues {
    pub covered_holes: f32,
    pub row_transitions: f32,
    pub column_transitions: f32,
    pub surface_roughness: f32,
    pub max_height: f32,
    pub deep_well_risk: f32,
    pub sum_of_heights: f32,
    pub lines_cleared: f32,
    pub i_well_reward: f32,
}

impl fmt::Debug for MetricValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            covered_holes,
            row_transitions,
            column_transitions,
            surface_roughness,
            max_height,
            deep_well_risk,
            sum_of_heights,
            lines_cleared,
            i_well_reward,
        } = *self;
        f.debug_struct("MetricValues")
            .field("covered_holes", &F32Fmt(covered_holes))
            .field("row_transitions", &F32Fmt(row_transitions))
            .field("column_transitions", &F32Fmt(column_transitions))
            .field("surface_roughness", &F32Fmt(surface_roughness))
            .field("max_height", &F32Fmt(max_height))
            .field("deep_well_risk", &F32Fmt(deep_well_risk))
            .field("sum_of_heights", &F32Fmt(sum_of_heights))
            .field("lines_cleared", &F32Fmt(lines_cleared))
            .field("i_well_reward", &F32Fmt(i_well_reward))
            .finish()
    }
}

struct F32Fmt(f32);
impl fmt::Debug for F32Fmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 0.123456789 -> 0.123_456_789
        let s = format!("{:?}", self.0);
        let (int, frac) = s.split_once('.').unwrap();
        write!(f, "{int}.")?;
        for (i, c) in frac.chars().enumerate() {
            if i > 0 && i % 3 == 0 {
                write!(f, "_")?;
            }
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

pub(crate) const METRIC_COUNT: usize = size_of::<MetricValues>() / size_of::<f32>();

impl MetricValues {
    pub(crate) const fn from_array(arr: [f32; METRIC_COUNT]) -> Self {
        Self {
            covered_holes: arr[0],
            row_transitions: arr[1],
            column_transitions: arr[2],
            surface_roughness: arr[3],
            max_height: arr[4],
            deep_well_risk: arr[5],
            sum_of_heights: arr[6],
            lines_cleared: arr[7],
            i_well_reward: arr[8],
        }
    }

    pub(crate) const fn to_array(&self) -> [f32; METRIC_COUNT] {
        [
            self.covered_holes,
            self.row_transitions,
            self.column_transitions,
            self.surface_roughness,
            self.max_height,
            self.deep_well_risk,
            self.sum_of_heights,
            self.lines_cleared,
            self.i_well_reward,
        ]
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Metrics(MetricValues);

impl Metrics {
    pub(crate) fn to_array(&self) -> [f32; METRIC_COUNT] {
        self.0.to_array()
    }

    pub(crate) fn measure(board: &BitBoard, placement: Piece) -> Self {
        let analysis = BoardAnalysis::from_board(board, placement);

        Self(MetricValues {
            covered_holes: CoveredHolesMetric.measure(&analysis).normalized,
            row_transitions: RowTransitionsMetric.measure(&analysis).normalized,
            column_transitions: ColumnTransitionsMetric.measure(&analysis).normalized,
            surface_roughness: SurfaceRoughnessMetric.measure(&analysis).normalized,
            max_height: MaxHeightMetric.measure(&analysis).normalized,
            deep_well_risk: DeepWellRiskMetric.measure(&analysis).normalized,
            sum_of_heights: SumOfHeightsMetric.measure(&analysis).normalized,
            lines_cleared: LineClearMetric.measure(&analysis).normalized,
            i_well_reward: IWellRewardMetric.measure(&analysis).normalized,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricSignal {
    Positive,
    Negative,
}

pub struct MetricValue {
    pub raw: u32,
    pub transformed: f32,
    pub normalized: f32,
}

pub trait MetricSource: fmt::Debug {
    const MAX_VALUE: f32;
    const SIGNAL: MetricSignal;

    fn name(&self) -> &'static str;
    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32;

    #[expect(clippy::cast_precision_loss)]
    fn transform(&self, raw: u32) -> f32 {
        raw as f32
    }

    fn normalize(&self, transformed: f32) -> f32 {
        let norm = (transformed / Self::MAX_VALUE).clamp(0.0, 1.0);
        match Self::SIGNAL {
            MetricSignal::Positive => norm,
            MetricSignal::Negative => 1.0 - norm,
        }
    }

    fn measure(&self, analysis: &BoardAnalysis) -> MetricValue {
        let raw = self.measure_raw(analysis);
        let transformed = self.transform(raw);
        let normalized = self.normalize(transformed);
        MetricValue {
            raw,
            transformed,
            normalized,
        }
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

    fn name(&self) -> &'static str {
        "Covered Holes"
    }

    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
        core::iter::zip(analysis.column_heights, analysis.column_occupied_cells)
            .map(|(h, occ)| u32::from(h - occ))
            .sum()
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(&self, raw: u32) -> f32 {
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

    fn name(&self) -> &'static str {
        "Row Transitions"
    }

    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
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

    fn name(&self) -> &'static str {
        "Column Transitions"
    }

    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
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

    fn name(&self) -> &'static str {
        "Surface Roughness"
    }

    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
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

    fn name(&self) -> &'static str {
        "Max Height"
    }

    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
        let max_height = *analysis.column_heights.iter().max().unwrap();
        u32::from(max_height)
    }

    fn transform(&self, raw: u32) -> f32 {
        const SAFE_THRESHOLD: f32 = 0.7;
        #[expect(clippy::cast_precision_loss)]
        let h = (raw as f32) / (BitBoard::PLAYABLE_HEIGHT as f32);
        if h <= SAFE_THRESHOLD {
            0.0
        } else {
            (h - SAFE_THRESHOLD) / (1.0 - SAFE_THRESHOLD)
        }
    }

    fn normalize(&self, norm: f32) -> f32 {
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

    fn name(&self) -> &'static str {
        "Deep Well Risk"
    }

    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
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

    fn normalize(&self, norm: f32) -> f32 {
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

    fn name(&self) -> &'static str {
        "Sum of Heights"
    }

    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
        analysis.column_heights.iter().map(|&h| u32::from(h)).sum()
    }
}

#[derive(Debug)]
pub struct IWellRewardMetric;

impl MetricSource for IWellRewardMetric {
    const MAX_VALUE: f32 = 1.0;
    const SIGNAL: MetricSignal = MetricSignal::Positive;

    fn name(&self) -> &'static str {
        "I-Well Reward"
    }

    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
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
    fn transform(&self, raw: u32) -> f32 {
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

    fn name(&self) -> &'static str {
        "Lines Cleared"
    }

    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32 {
        u32::try_from(analysis.cleared_lines).unwrap()
    }

    fn transform(&self, raw: u32) -> f32 {
        const WEIGHT: [f32; 5] = [0.0, 0.0, 1.0, 2.0, 6.0];
        WEIGHT[usize::try_from(raw).unwrap()]
    }
}
