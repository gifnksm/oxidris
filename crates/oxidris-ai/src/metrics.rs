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
    &LineClearRewardMetric,
    &IWellRewardMetric,
]);

pub(crate) const ALL_METRICS_COUNT: usize = ALL_METRICS.0.len();

#[derive(Debug, Clone)]
pub struct MetricSet<'a, const N: usize>([&'a dyn DynMetricSource; N]);

impl<'a, const N: usize> MetricSet<'a, N> {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        N == 0
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        N
    }

    #[must_use]
    pub const fn as_array(&self) -> [&'a dyn DynMetricSource; N] {
        self.0
    }

    #[must_use]
    pub fn measure(&self, board: &BitBoard, placement: Piece) -> [MetricMeasurement; N] {
        let analysis = BoardAnalysis::from_board(board, placement);
        self.0.map(|metric| metric.measure(&analysis))
    }

    #[must_use]
    pub fn measure_normalized(&self, board: &BitBoard, placement: Piece) -> [f32; N] {
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
pub struct MetricMeasurement {
    pub raw: u32,
    pub transformed: f32,
    pub normalized: f32,
}

pub trait MetricSource: fmt::Debug {
    const NORMALIZATION_CAP: f32;
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
        let norm = (transformed / Self::NORMALIZATION_CAP).clamp(0.0, 1.0);
        match Self::SIGNAL {
            MetricSignal::Positive => norm,
            MetricSignal::Negative => 1.0 - norm,
        }
    }

    #[must_use]
    fn measure(analysis: &BoardAnalysis) -> MetricMeasurement {
        let raw = Self::measure_raw(analysis);
        let transformed = Self::transform(raw);
        let normalized = Self::normalize(transformed);
        MetricMeasurement {
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
    fn normalization_cap(&self) -> f32;
    #[must_use]
    fn signal(&self) -> MetricSignal;
    #[must_use]
    fn measure_raw(&self, analysis: &BoardAnalysis) -> u32;
    #[must_use]
    fn transform(&self, raw: u32) -> f32;
    #[must_use]
    fn normalize(&self, transformed: f32) -> f32;
    #[must_use]
    fn measure(&self, analysis: &BoardAnalysis) -> MetricMeasurement;
}

impl<T> DynMetricSource for T
where
    T: MetricSource,
{
    fn name(&self) -> &'static str {
        T::name()
    }

    fn normalization_cap(&self) -> f32 {
        T::NORMALIZATION_CAP
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

    fn measure(&self, analysis: &BoardAnalysis) -> MetricMeasurement {
        T::measure(analysis)
    }
}

#[derive(Debug)]
pub struct CoveredHolesMetric;

// Covered holes are empty cells with at least one block above them.
// They are one of the strongest losing factors in Tetris,
// because they are difficult or impossible to clear directly.
//
// Empirical distribution (weak–mid AI, long-run):
//   Median ≈ 1
//   P90    ≈ 4
//   P95    ≈ 6
//   P99    ≈ 9
//
// Interpretation (raw hole count):
//   0       : ideal, fully clean board
//   1–2     : early warning, still recoverable
//   3–4     : dangerous, recovery is hard
//   5–6     : near-losing position
//   7+      : effectively lost
//
// A power transform (holes^1.7) strongly emphasizes early hole creation,
// while saturating quickly for already-losing boards.
// The normalization cap (~12) corresponds to ~6 practical holes.
impl MetricSource for CoveredHolesMetric {
    /// Normalization cap chosen from empirical distribution:
    /// P95 ≈ 6 holes → transformed ≈ 6^1.7 ≈ 21.03
    const NORMALIZATION_CAP: f32 = 21.03;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Covered Holes"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        // For each column:
        //   covered_holes = column_height - number_of_occupied_cells
        // This counts empty cells that have at least one block above.
        core::iter::zip(analysis.column_heights, analysis.column_occupied_cells)
            .map(|(h, occ)| u32::from(h - occ))
            .sum()
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        (raw as f32).powf(1.7)
    }
}

#[derive(Debug)]
pub struct RowTransitionsMetric;

// Row transitions measure horizontal fragmentation by counting
// occupancy changes between adjacent cells within each row.
//
// Only transitions *inside* the playable area are counted.
// Board walls are intentionally ignored to preserve left–right symmetry
// and avoid artificial incentives for edge stacking.
//
// This metric penalizes:
//   - horizontally fragmented structures
//   - narrow gaps and broken rows
//   - layouts that are inefficient to clear
//
// Empirical distribution (10x20 board, weak–mid AI):
//   Median ≈ 11
//   P90    ≈ 26
//   P95    ≈ 32
//   P99    ≈ 44
//
// Interpretation (raw):
//   0–15    : very clean and flat surface
//   20–30   : normal mid-game roughness
//   30–40   : dangerous fragmentation
//   40+     : highly unstable board
//
// This is a negative metric; lower transition counts are better.
impl MetricSource for RowTransitionsMetric {
    /// Normalization cap chosen from empirical P95 (~32)
    const NORMALIZATION_CAP: f32 = 32.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Row Transitions"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        let mut transitions = 0;
        for row in analysis.board.playable_rows() {
            let mut cells = row.iter_playable_cells();
            let mut prev_occupied = cells.next().unwrap();
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
// scanning from the top of the playable area downwards.
//
// This metric measures vertical fragmentation inside columns,
// including stacked blocks and covered holes.
//
// Observed distribution (10x20 board, self-play sampling):
//   Mean   ≈ 11
//   Median ≈ 11
//   P90    ≈ 17
//   P95    ≈ 19
//   P99    ≈ 25
//   Max    ≈ 45
//
// Interpretation:
//   0-15   : clean columns, mostly solid
//   15–25  : typical mid-game vertical fragmentation
//   25+    : severe internal instability (many splits / holes)
impl MetricSource for ColumnTransitionsMetric {
    // Normalization cap is set to the observed P95 value.
    // Values beyond this represent already-losing positions
    // and are saturated to preserve signal resolution
    // in the normal play regime.
    const NORMALIZATION_CAP: f32 = 19.0;
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
        }
        transitions
    }
}

#[derive(Debug)]
pub struct SurfaceRoughnessMetric;

// Surface roughness measures local curvature of the board surface
// using second-order height differences (discrete Laplacian).
//
// This metric captures small-scale unevenness that may not
// immediately create holes, but increases future instability.
//
// Observed distribution (10x20 board, self-play sampling):
//   Mean   ≈ 15
//   Median ≈ 12
//   P90    ≈ 28
//   P95    ≈ 37
//   P99    ≈ 55
//   Max    ≈ 130
//
// Interpretation:
//   < 10   : flat or intentionally shaped surface
//   10–30  : normal mid-game roughness
//   30–55  : highly uneven, high hole risk
//   > 55   : chaotic surface, often unrecoverable
//
// This metric complements row and column transitions
// by remaining sensitive even when the overall stack is low.
impl MetricSource for SurfaceRoughnessMetric {
    // Normalization cap is set to the observed P95 value.
    // Values beyond this correspond to highly chaotic surfaces
    // and are saturated to preserve resolution in normal play.
    const NORMALIZATION_CAP: f32 = 37.0;
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

// Max Height represents imminent top-out danger.
//
// Unlike other metrics, max height is intentionally ignored
// for most of the game and only penalized near the ceiling,
// reflecting the irreversible nature of top-out.
//
// Observed distribution (10x20 board, self-play sampling):
//   Mean   ≈ 5.7
//   Median ≈ 4
//   P90    ≈ 12
//   P95    ≈ 12
//   P99    ≈ 16
//   Max    = 20 (top-out)
//
// Interpretation:
//   <= 10  : safe, no penalty applied
//   13–15  : dangerous zone, recovery still possible
//   >= 16  : critical, near-certain top-out
//
// A linear ramp is applied above the safe threshold,
// followed by an exponential penalty to strongly discourage
// states close to top-out.
//
// This metric acts as a hard constraint rather than
// a general board quality measure.
impl MetricSource for MaxHeightMetric {
    // Normalization cap applies to the transformed value (0–1),
    // not to the raw board height.
    const NORMALIZATION_CAP: f32 = 0.18;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Max Height"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        let max_height = *analysis.column_heights.iter().max().unwrap();
        u32::from(max_height)
    }

    fn transform(raw: u32) -> f32 {
        // Heights up to this threshold are considered safe
        // and intentionally ignored.
        const SAFE_THRESHOLD: f32 = 0.5; // ≈ height 10 on a 20-row board
        #[expect(clippy::cast_precision_loss)]
        let h = (raw as f32) / (BitBoard::PLAYABLE_HEIGHT as f32);
        if h <= SAFE_THRESHOLD {
            0.0
        } else {
            // normalized danger ∈ [0, 1]
            let danger = (h - SAFE_THRESHOLD) / (1.0 - SAFE_THRESHOLD);
            // exponential escalation near ceiling
            danger.powf(2.5)
        }
    }
}

#[derive(Debug)]
pub struct DeepWellRiskMetric;

// Deep Well Risk detects excessively deep single-column wells
// that are difficult or impossible to recover from.
//
// Only wells deeper than 2 are considered dangerous.
// Shallow wells (depth <= 2) are allowed to preserve freedom
// for controlled I-well construction.
//
// Raw value definition:
//   raw = Σ (max(depth - 2, 0)^2)
//
// Squaring aggressively penalizes over-committed vertical wells,
// reflecting the non-linear difficulty of recovery.
//
// Observed distribution (10x20 board, self-play sampling):
//   Median ≈ 0
//   P90    ≈ 34
//   P95    ≈ 59
//   P99    ≈ 136
//   Max    ≫ 100 (rare catastrophic outliers)
//
// Interpretation:
//   raw ≈ 0        : safe or controlled wells only
//   raw ≈ 30–60    : dangerous but sometimes recoverable
//   raw ≥ 100      : near-fatal vertical structure
//
// This metric is strictly a safety penalty.
// It does NOT reward I-wells and should be combined with
// a separate positive I-well reward metric.
//
// Normalization is capped at P95 to preserve resolution
// between dangerous and fatal states.
impl MetricSource for DeepWellRiskMetric {
    // Cap is based on P95 to distinguish dangerous vs fatal wells.
    // ln(1+59) ≈ 4.09
    const NORMALIZATION_CAP: f32 = 4.09;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Deep Well Risk"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        analysis
            .column_well_depths
            .iter()
            // Allow shallow wells for I-well construction
            .filter(|depth| **depth > 2)
            .map(|depth| {
                let depth = u32::from(*depth - 2);
                depth * depth
            })
            .sum()
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        // Exponential growth models non-linear recovery difficulty
        (raw as f32).ln_1p()
    }
}

#[derive(Debug)]
pub struct SumOfHeightsMetric;

// Sum of Heights measures overall board pressure by summing
// all column heights.
//
// This metric captures *global stacking pressure* that is
// not necessarily reflected by local roughness or transitions.
// Unlike Max Height, it penalizes gradual accumulation of blocks
// across the entire board.
//
// Empirical distribution (10x20 board, weak–mid AI):
//   Median ≈  27
//   P90    ≈  83
//   P95    ≈  93
//   P99    ≈ 122
//
// Typical ranges (10x20 board, empirical):
//   0–40    : very safe, early-game state
//   40–80   : normal mid-game pressure
//   80–120  : high pressure, limited recovery options
//   120+    : near top-out or effectively lost
//
// NORMALIZATION_CAP is set to the empirical P99 (~120),
// ignoring extreme terminal states while preserving sensitivity
// throughout practical play.
//
// This metric follows the same linear-raw philosophy as
// Row Transitions and Column Transitions.
impl MetricSource for SumOfHeightsMetric {
    /// Normalization cap chosen from empirical P95 (93)
    /// ln(1+9.3) ≈ 2.33
    const NORMALIZATION_CAP: f32 = 2.33;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Sum of Heights"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        analysis.column_heights.iter().map(|&h| u32::from(h)).sum()
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        (raw as f32 / 10.0).ln_1p()
    }
}

#[derive(Debug)]
pub struct LineClearRewardMetric;

// Lines cleared represent forward progress and efficiency.
// Weights strongly favor tetrises (4-line clears).
impl MetricSource for LineClearRewardMetric {
    const NORMALIZATION_CAP: f32 = 6.0;
    const SIGNAL: MetricSignal = MetricSignal::Positive;

    fn name() -> &'static str {
        "Lines Clear Reward"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        u32::try_from(analysis.cleared_lines).unwrap()
    }

    fn transform(raw: u32) -> f32 {
        const WEIGHT: [f32; 5] = [0.0, 0.5, 1.0, 2.0, 6.0];
        WEIGHT[usize::try_from(raw).unwrap()]
    }
}

#[derive(Debug)]
pub struct IWellRewardMetric;

impl MetricSource for IWellRewardMetric {
    const NORMALIZATION_CAP: f32 = 1.0;
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
