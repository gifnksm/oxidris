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

/// Penalizes covered holes (empty cells with blocks above).
///
/// This metric penalizes:
///
/// - Early hole creation
/// - Unrecoverable board states
/// - Cells that are difficult or impossible to clear
///
/// # Raw measurement
///
/// - For each column, compute `column_height - occupied_cells`; sum across columns.
/// - Counts empty cells that have at least one block above them.
///
/// # Stats (raw values, 10x20, weak–mid AI, long run)
///
/// - Mean ≈ 1.59
/// - P10 = 0
/// - P25 = 0
/// - Median = 1
/// - P75 = 2
/// - P90 = 4
/// - P95 = 6
/// - P99 = 9
///
/// # Interpretation (raw hole count)
///
/// - 0: ideal (≤P25)
/// - 1: good, minor concern (P25-Median)
/// - 2-4: moderate risk (Median-P90)
/// - 5-6: high risk (P90-P95)
/// - 7+: critical (P95+)
///
/// # Normalization
///
/// - `NORMALIZATION_CAP` = 6.0 (set to P95; linear, uses raw directly).
/// - `SIGNAL` = Negative (fewer holes is better).
#[derive(Debug)]
pub struct CoveredHolesMetric;

impl MetricSource for CoveredHolesMetric {
    const NORMALIZATION_CAP: f32 = 6.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Covered Holes"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        core::iter::zip(analysis.column_heights, analysis.column_occupied_cells)
            .map(|(h, occ)| u32::from(h - occ))
            .sum()
    }
}

/// Penalizes horizontal fragmentation by counting occupancy changes between adjacent cells within each row.
///
/// This metric penalizes:
///
/// - Horizontally fragmented structures
/// - Narrow gaps and broken rows
/// - Layouts that are inefficient to clear
///
/// # Raw measurement
///
/// - For each row, scan left to right within the playable area only.
/// - Count transitions where adjacent cells differ in occupancy (empty ↔ filled).
/// - Board walls are intentionally ignored to preserve left–right symmetry and avoid artificial incentives for edge stacking.
///
/// This differs from typical implementations that treat walls as filled cells, which can bias AI toward center placement.
/// By excluding walls, this metric evaluates edge and center placements fairly.
///
/// # Stats (raw values, 10x20, weak–mid AI)
///
/// - Mean ≈ 13.70
/// - P10 = 5
/// - P25 = 7
/// - Median = 11
/// - P75 = 18
/// - P90 = 26
/// - P95 = 32
/// - P99 = 44
///
/// # Interpretation (raw transition count)
///
/// - 0-7: very clean (≤P25)
/// - 8-11: clean (P25-Median)
/// - 12-18: normal mid-game (Median-P75)
/// - 19-26: increased fragmentation (P75-P90)
/// - 27-32: high fragmentation (P90-P95)
/// - 33+: critical instability (P95+)
///
/// # Normalization
///
/// - `NORMALIZATION_CAP` = 32.0 (set to P95; linear, uses raw directly).
/// - `SIGNAL` = Negative (fewer transitions is better).
#[derive(Debug)]
pub struct RowTransitionsMetric;

impl MetricSource for RowTransitionsMetric {
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

/// Penalizes vertical fragmentation within columns by counting occupancy changes from top to bottom.
///
/// This metric penalizes:
///
/// - Vertical fragmentation inside columns
/// - Internal splits and covered holes
/// - Unstable stacking structures
///
/// # Raw measurement
///
/// - For each column, scan from top to bottom within the playable area.
/// - Count transitions where adjacent cells differ in occupancy (empty ↔ filled).
///
/// # Stats (raw values, 10x20, self-play sampling)
///
/// - Mean ≈ 11.41
/// - P10 = 8
/// - P25 = 9
/// - Median = 11
/// - P75 = 13
/// - P90 = 17
/// - P95 = 19
/// - P99 = 25
/// - Max = 45
///
/// # Interpretation (raw transition count)
///
/// - 0-9: very clean (≤P25)
/// - 10-11: clean (P25-Median)
/// - 12-13: normal (Median-P75)
/// - 14-17: increased fragmentation (P75-P90)
/// - 18-19: high fragmentation (P90-P95)
/// - 20+: severe instability (P95+)
///
/// # Normalization
///
/// - `NORMALIZATION_CAP` = 19.0 (set to P95; linear, uses raw directly).
/// - `SIGNAL` = Negative (fewer transitions is better).
#[derive(Debug)]
pub struct ColumnTransitionsMetric;

impl MetricSource for ColumnTransitionsMetric {
    const NORMALIZATION_CAP: f32 = 19.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Column Transitions"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        let mut transitions = 0;
        for x in BitBoard::PLAYABLE_X_RANGE {
            let mut prev_occupied = analysis.board.playable_row(0).is_cell_occupied(x);
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

/// Penalizes local surface curvature using second-order height differences (discrete Laplacian).
///
/// This metric penalizes:
///
/// - Small-scale surface unevenness
/// - Local height variations that increase future instability
/// - Shapes that are prone to creating holes
///
/// Complements row and column transitions by remaining sensitive even when the overall stack is low.
///
/// # Raw measurement
///
/// - For each triplet of adjacent columns, compute the discrete Laplacian: `|(right - mid) - (mid - left)|`.
/// - Sum across all triplets.
///
/// # Stats (raw values, 10x20, self-play sampling)
///
/// - Mean ≈ 15.04
/// - P10 = 6
/// - P25 = 8
/// - Median = 12
/// - P75 = 18
/// - P90 = 28
/// - P95 = 37
/// - P99 = 55
/// - Max = 130
///
/// # Interpretation (raw roughness)
///
/// - 0-8: flat surface (≤P25)
/// - 9-12: smooth (P25-Median)
/// - 13-18: normal roughness (Median-P75)
/// - 19-28: increased unevenness (P75-P90)
/// - 29-37: high roughness (P90-P95)
/// - 38+: critical chaos (P95+)
///
/// # Normalization
///
/// - `NORMALIZATION_CAP` = 37.0 (set to P95; linear, uses raw directly).
/// - `SIGNAL` = Negative (flatter surface is better).
#[derive(Debug)]
pub struct SurfaceRoughnessMetric;

impl MetricSource for SurfaceRoughnessMetric {
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

/// Penalizes imminent top-out danger based on maximum column height.
///
/// This metric penalizes:
///
/// - Approaching the ceiling (irreversible top-out risk)
/// - States close to game over
///
/// Unlike other metrics, max height is intentionally ignored for most of the game and only penalized near the ceiling,
/// reflecting the irreversible nature of top-out. Acts as a hard constraint rather than a general board quality measure.
///
/// # Raw measurement
///
/// - `raw = max(column_heights)`: the tallest column on the board.
///
/// # Transform
///
/// - Heights up to `SAFE_THRESHOLD` (0.5 ≈ height 10 on 20-row board) are considered safe and map to 0.0.
/// - Above the threshold, apply exponential escalation: `danger^2.5` where `danger = (h - SAFE_THRESHOLD) / (1.0 - SAFE_THRESHOLD)`.
/// - This strongly discourages states close to top-out.
///
/// # Stats (10x20, self-play sampling)
///
/// Raw values:
///
/// - Mean ≈ 5.66
/// - P10 = 2
/// - P25 = 3
/// - Median = 4
/// - P75 = 8
/// - P90 = 12
/// - P95 = 12
/// - P99 = 16
/// - Max = 20 (top-out)
///
/// Transformed values:
///
/// - P75 = 0.00 (safe zone)
/// - P90 = 0.02
/// - P95 = 0.02
/// - P99 = 0.28
///
/// # Interpretation (raw height)
///
/// - 0-10: safe, no penalty applied
/// - 11-12: caution zone
/// - 13-15: dangerous zone, recovery still possible
/// - 16+: critical, near-certain top-out
///
/// # Normalization
///
/// - `NORMALIZATION_CAP` = 0.18 (applies to transformed value, not raw height).
/// - `SIGNAL` = Negative (lower height is better).
#[derive(Debug)]
pub struct MaxHeightMetric;

impl MetricSource for MaxHeightMetric {
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
        const SAFE_THRESHOLD: f32 = 0.5;
        #[expect(clippy::cast_precision_loss)]
        let h = (raw as f32) / (BitBoard::PLAYABLE_HEIGHT as f32);
        if h <= SAFE_THRESHOLD {
            0.0
        } else {
            let danger = (h - SAFE_THRESHOLD) / (1.0 - SAFE_THRESHOLD);
            danger.powf(2.5)
        }
    }
}

/// Penalizes excessively deep single-column wells that are difficult or impossible to recover from.
///
/// This metric penalizes:
///
/// - Over-committed vertical wells
/// - Single columns with extreme depth
/// - Structures with non-linear recovery difficulty
///
/// Only wells deeper than 2 are considered dangerous. Shallow wells (depth ≤ 2) are allowed to preserve freedom
/// for controlled I-well construction. This metric is strictly a safety penalty and does NOT reward I-wells;
/// combine with `IWellRewardMetric` for balanced evaluation.
///
/// # Raw measurement
///
/// - `raw = Σ (max(depth - 2, 0)^2)` across all columns.
/// - Squaring aggressively penalizes over-committed vertical wells, reflecting non-linear recovery difficulty.
///
/// # Transform
///
/// - `ln(1 + raw)`: logarithmic transform models exponential growth in recovery difficulty.
///
/// # Stats (10x20, self-play sampling)
///
/// Raw values:
/// - Mean ≈ 10.49
/// - P10 = 0
/// - P25 = 0
/// - Median = 0
/// - P75 = 5
/// - P90 = 34
/// - P95 = 59
/// - P99 = 136
///
/// Transformed values:
/// - P75 ≈ 1.79
/// - P90 ≈ 3.56
/// - P95 ≈ 4.09
/// - P99 ≈ 4.92
///
/// # Interpretation (raw)
///
/// - 0: safe, no deep wells (≤Median)
/// - 1-5: controlled wells (Median-P75)
/// - 6-34: risky depth (P75-P90)
/// - 35-59: dangerous (P90-P95)
/// - 60+: near-fatal vertical structure (P95+)
///
/// # Normalization
///
/// - `NORMALIZATION_CAP` = 4.09 (set to P95 transformed value: ln(1+59) ≈ 4.09).
/// - `SIGNAL` = Negative (shallower wells is better).
#[derive(Debug)]
pub struct DeepWellRiskMetric;

impl MetricSource for DeepWellRiskMetric {
    const NORMALIZATION_CAP: f32 = 4.09;
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

    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        (raw as f32).ln_1p()
    }
}

/// Penalizes global stacking pressure by summing all column heights.
///
/// This metric penalizes:
///
/// - Gradual accumulation of blocks across the entire board
/// - Overall board pressure not captured by local roughness or transitions
/// - High average stack height
///
/// Unlike `MaxHeightMetric`, which focuses on top-out danger from the tallest column, this metric captures
/// cumulative pressure across all columns. It reflects the total "weight" of the board state.
///
/// # Raw measurement
///
/// - `raw = Σ (column_heights)` across all 10 columns.
/// - Linear accumulation of individual column heights.
///
/// # Stats (raw values, 10x20, self-play sampling)
///
/// - Mean ≈ 40.02
/// - P10 = 14
/// - P25 = 14
/// - Median ≈ 27
/// - P75 = 58
/// - P90 ≈ 83
/// - P95 ≈ 93
/// - P99 ≈ 122
///
/// # Interpretation (raw)
///
/// - 0-14: very low pressure (≤P25)
/// - 15-27: low pressure, early-game state (P25-Median)
/// - 28-58: moderate pressure (Median-P75)
/// - 59-83: elevated pressure (P75-P90)
/// - 84-93: high pressure, limited recovery options (P90-P95)
/// - 94+: near top-out danger (P95+)
///
/// # Normalization
///
/// - `NORMALIZATION_CAP` = 93.0 (set to P95; linear, uses raw directly).
/// - `SIGNAL` = Negative (lower total height is better).
#[derive(Debug)]
pub struct SumOfHeightsMetric;

impl MetricSource for SumOfHeightsMetric {
    const NORMALIZATION_CAP: f32 = 93.0;
    const SIGNAL: MetricSignal = MetricSignal::Negative;

    fn name() -> &'static str {
        "Sum of Heights"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        analysis.column_heights.iter().map(|&h| u32::from(h)).sum()
    }
}

/// Rewards line clears with strong emphasis on efficient 4-line clears (tetrises).
///
/// This metric encourages:
///
/// - Clearing multiple lines in a single placement
/// - Prioritizing tetrises (4-line clears) over singles/doubles
/// - Forward progress and board cleanup
///
/// The reward structure strongly favors tetrises to align with optimal Tetris strategy, where
/// maximizing 4-line clears yields both higher scores and better board states.
///
/// # Raw measurement
///
/// - `raw = number of lines cleared` (0-4).
/// - Direct count from the placement result.
///
/// # Transform
///
/// - Discrete mapping: `[0→0.0, 1→0.0, 2→1.0, 3→2.0, 4→6.0]`
/// - Singles (1 line) are not rewarded to discourage inefficient clearing.
/// - Tetrises receive 6× the reward of doubles, reflecting strategic importance.
///
/// # Stats
///
/// This is a per-placement reward, not a cumulative board state metric. Distribution depends on
/// play style and board construction strategy.
///
/// # Interpretation (raw)
///
/// - 0-1: no reward (inefficient or no clear)
/// - 2: minor reward (double clear)
/// - 3: moderate reward (triple clear)
/// - 4: major reward (tetris)
///
/// # Normalization
///
/// - `NORMALIZATION_CAP` = 6.0 (maximum transformed value for tetris).
/// - `SIGNAL` = Positive (more lines cleared is better).
#[derive(Debug)]
pub struct LineClearRewardMetric;

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
        const WEIGHT: [f32; 5] = [0.0, 0.0, 1.0, 2.0, 6.0];
        WEIGHT[usize::try_from(raw).unwrap()]
    }
}

/// Encourages maintaining an edge I-well for reliable tetrises without over-committing.
///
/// This metric encourages:
///
/// - Building a single-column well at the board edge
/// - Maintaining tetris-ready depth (around 4)
/// - Immediate consumption when I-piece is available
///
/// Considers only the leftmost and rightmost columns; center wells are ignored.
///
/// # Raw measurement
///
/// - `raw = max(left_well_depth, right_well_depth)`.
/// - If a ready I-well (`depth >= 4`) coincides with placing an `I` piece,
///   set `raw = 0` to avoid double rewarding and to encourage an immediate tetris.
///
/// # Transform
///
/// - Gaussian centered at depth 4 with `σ = 2`: $f(d) = e^{-(d-4)^2/(2\sigma^2)}$.
/// - Rewards tetris-ready wells most; decays for wells that are too shallow or too deep.
///
/// # Normalization
///
/// - `NORMALIZATION_CAP` = 1.0. Output naturally lies in $[0, 1]$.
/// - `SIGNAL` = Positive (higher is better).
///
/// # Interpretation (edge well depth)
///
/// - 0–1: negligible reward (no/very shallow well)
/// - 2–3: moderate reward (well construction in progress)
/// - 4: peak reward (ideal tetris-ready well)
/// - 5+: decreasing reward (over-commitment discouraged)
///
/// # Rationale and interplay
///
/// - Complements `DeepWellRiskMetric` by penalizing excessive vertical wells.
/// - Synergizes with `LineClearRewardMetric` to favor consistent tetrises.
/// - The consumption guard discourages hoarding when an `I` piece is available.
#[derive(Debug)]
pub struct IWellRewardMetric;

impl MetricSource for IWellRewardMetric {
    const NORMALIZATION_CAP: f32 = 1.0;
    const SIGNAL: MetricSignal = MetricSignal::Positive;

    fn name() -> &'static str {
        "I-Well Reward"
    }

    fn measure_raw(analysis: &BoardAnalysis) -> u32 {
        let left_well_depth = analysis.column_well_depths[0];
        let right_well_depth = analysis.column_well_depths[BitBoard::PLAYABLE_WIDTH - 1];
        let max_depth = u8::max(left_well_depth, right_well_depth);

        if max_depth >= 4 && analysis.placement.kind() == PieceKind::I {
            return 0;
        }

        u32::from(max_depth)
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        let peak = 4.0;
        let sigma = 2.0;
        (-(raw as f32 - peak).powi(2) / (2.0 * sigma * sigma)).exp()
    }
}
