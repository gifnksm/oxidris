//! Normalized evaluation features for Tetris board states.
//!
//! Provides board evaluation features for Tetris, each producing a normalized score in \[0.0, 1.0\].
//! Higher is always better after normalization; negative features are inverted via [`FeatureSignal::Negative`].
//!
//! # Typology
//!
//! Features follow a naming convention based on their behavior:
//!
//! - **Risk**: Thresholded danger that escalates rapidly beyond a safe limit (e.g., [`DeepWellRisk`], [`TopOutRisk`])
//! - **Penalty**: Smooth negative signals (e.g., [`HolesPenalty`], [`HoleDepthPenalty`], [`RowTransitionsPenalty`])
//! - **Reward**: Smooth positive signals (e.g., [`IWellReward`])
//! - **Bonus**: Discrete strong rewards (e.g., [`LineClearBonus`])
//!
//! # Feature Categories
//!
//! Features are categorized by their role in gameplay:
//!
//! ## Survival Features
//!
//! Directly affect game termination (when the game ends):
//!
//! - [`HolesPenalty`] - Number of holes (empty cells with blocks above)
//! - [`HoleDepthPenalty`] - Sum of depths of all holes
//! - [`MaxHeightPenalty`] - Maximum column height
//! - [`TotalHeightPenalty`] - Sum of all column heights
//! - [`CenterColumnsPenalty`] - Sum of center column heights (columns 3-6)
//! - [`TopOutRisk`] - Risk of topping out (height-based threshold)
//! - [`CenterTopOutRisk`] - Risk of topping out in center columns
//!
//! ## Structure Features
//!
//! Affect placement flexibility and future options:
//!
//! - [`SurfaceBumpinessPenalty`] - Sum of absolute height differences between adjacent columns
//! - [`SurfaceRoughnessPenalty`] - Variance in column heights
//! - [`RowTransitionsPenalty`] - Number of horizontal empty-to-filled transitions
//! - [`ColumnTransitionsPenalty`] - Number of vertical empty-to-filled transitions
//! - [`WellDepthPenalty`] - Depth of deepest well (reduces placement flexibility)
//! - [`DeepWellRisk`] - Risk from excessively deep wells (reduces recovery options)
//!
//! ## Score Features
//!
//! Directly contribute to game score:
//!
//! - [`LineClearBonus`] - Number of lines cleared by this placement
//! - [`IWellReward`] - Quality of I-piece well setup (depth ~4 is optimal)
//!
//! # Feature Processing Pipeline
//!
//! Each feature goes through a three-step pipeline:
//!
//! 1. **Extract Raw** - Extract raw value from board state (e.g., count holes)
//! 2. **Transform** - Transform raw value into meaningful representation
//! 3. **Normalize** - Scale to \[0.0, 1.0\] using P05-P95 percentiles
//!
//! ## Transformation
//!
//! Most features use linear transformation (`raw as f32`), but some use custom transformations:
//!
//! - [`LineClearBonus`]: Exponential weighting (4-line Tetris gets 6× weight)
//! - [`IWellReward`]: Triangular peak centered at depth 4 (optimal for I-pieces)
//!
//! ## Normalization
//!
//! Normalization ranges vary by feature type:
//!
//! - **Penalty features** use P05-P95 range (smooth penalties across full observed range)
//! - **Risk features** use P75-P95 range (threshold-based, ignoring safe lower values)
//! - **Bonus/Reward features** use fixed ranges (e.g., 0.0-6.0, 0.0-1.0) based on transform output
//!
//! For percentile-based features:
//!
//! - Percentiles are computed from actual gameplay data and stored in the `stats` module (auto-generated)
//! - Values outside the percentile range are clipped to \[0.0, 1.0\]
//!
//! All negative features are inverted (lower raw values → higher normalized scores).
//!
//! # Design Decisions
//!
//! ## Why Percentile-Based Normalization?
//!
//! - **Data-driven**: Grounded in actual gameplay behavior
//! - **Robust to outliers**: P05-P95 range clips extremes
//! - **Simple and fast**: Linear scaling, no complex computation
//!
//! ## Current Limitations
//!
//! ### Linear Transformation for Survival Features
//!
//! Most survival features use linear transformation, which doesn't capture the non-linear
//! relationship between feature values and survival time. For example, the first hole has
//! much greater impact on survival than the 11th hole, but linear transformation treats
//! `holes: 0→1` the same as `holes: 10→11`.
//!
//! ### Feature Redundancy
//!
//! The feature set has two types of redundancy issues:
//!
//! 1. **Duplicate features with different normalization ranges**:
//!
//!    - `TopOutRisk` vs `MaxHeightPenalty` - Both measure maximum height but with different normalization
//!    - `CenterTopOutRisk` vs `CenterColumnsPenalty` - Both measure center column heights
//!    - `DeepWellRisk` vs `WellDepthPenalty` - Both measure well depth
//!
//!    These duplicates exist as an ad-hoc attempt to capture non-linearity through different
//!    scaling ranges, suggesting a systematic need for non-linear transformations.
//!
//! 2. **Similar features measuring overlapping properties**:
//!
//!    - `HolesPenalty` vs `HoleDepthPenalty` - Both measure holes (count vs depth)
//!    - `SurfaceBumpinessPenalty` vs `SurfaceRoughnessPenalty` - Both measure surface irregularity
//!    - `RowTransitionsPenalty` vs `ColumnTransitionsPenalty` - Both measure board complexity
//!
//!    These similar features may provide different perspectives on the same underlying property,
//!    but the redundancy creates several issues:
//!
//!    - **Training difficulty**: The genetic algorithm must learn weights for correlated features,
//!      which can lead to unstable or arbitrary weight assignments
//!    - **Interpretability**: Hard to determine which features are truly important when multiple
//!      features measure similar properties
//!    - **Computational cost**: Extra feature computation and evaluation overhead
//!
//! Note: Type 1 redundancy (duplicate features with different ranges) is intentional—an ad-hoc
//! workaround for the linear transformation limitation. Type 2 redundancy (overlapping features)
//! may or may not be beneficial and warrants investigation.
//!
//! # Usage
//!
//! Features are typically used through [`PlacementEvaluator`](crate::placement_evaluator::PlacementEvaluator):
//!
//! ```rust,no_run
//! use oxidris_evaluator::board_feature::ALL_BOARD_FEATURES;
//! use oxidris_evaluator::placement_evaluator::FeatureBasedPlacementEvaluator;
//!
//! // Create evaluator with all features
//! let features = ALL_BOARD_FEATURES.to_vec();
//! let weights = vec![1.0; features.len()];
//! let evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
//! ```
//!
//! Individual features can be computed directly using the trait methods:
//!
//! ```rust,ignore
//! use oxidris_evaluator::board_feature::{BoardFeatureSource, HolesPenalty};
//!
//! let raw = HolesPenalty::extract_raw(&analysis);
//! let transformed = HolesPenalty::transform(raw);
//! let normalized = HolesPenalty::normalize(transformed);
//! ```
//!
//! The [`BoardFeatureSource`] trait defines the feature interface:
//!
//! ```rust,ignore
//! pub trait BoardFeatureSource {
//!     const ID: &str;
//!     const NAME: &str;
//!     const NORMALIZATION_MIN: f32;
//!     const NORMALIZATION_MAX: f32;
//!     const SIGNAL: FeatureSignal;
//!
//!     fn extract_raw(analysis: &PlacementAnalysis) -> u32;
//!     fn transform(raw: u32) -> f32;  // Default: raw as f32
//!     fn normalize(transformed: f32) -> f32;
//! }
//! ```
//!
//! [`ALL_BOARD_FEATURES`] provides access to all active features for batch processing.

use std::fmt;

use crate::placement_analysis::PlacementAnalysis;

mod stats;

pub const ALL_BOARD_FEATURES: &[&dyn DynBoardFeatureSource] = &[
    &HolesPenalty,
    &HoleDepthPenalty,
    &RowTransitionsPenalty,
    &ColumnTransitionsPenalty,
    &SurfaceBumpinessPenalty,
    &SurfaceRoughnessPenalty,
    &WellDepthPenalty,
    &DeepWellRisk,
    &MaxHeightPenalty,
    &CenterColumnsPenalty,
    &CenterTopOutRisk,
    &TopOutRisk,
    &TotalHeightPenalty,
    &LineClearBonus,
    &IWellReward,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureSignal {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy)]
pub struct BoardFeatureValue {
    pub raw: u32,
    pub transformed: f32,
    pub normalized: f32,
}

pub trait BoardFeatureSource: fmt::Debug + Send + Sync {
    const ID: &str;
    const NAME: &str;
    const NORMALIZATION_MIN: f32;
    const NORMALIZATION_MAX: f32;
    const SIGNAL: FeatureSignal;

    #[must_use]
    fn extract_raw(analysis: &PlacementAnalysis) -> u32;

    #[must_use]
    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        raw as f32
    }

    #[must_use]
    fn normalize(transformed: f32) -> f32 {
        let span = Self::NORMALIZATION_MAX - Self::NORMALIZATION_MIN;
        let norm = ((transformed - Self::NORMALIZATION_MIN) / span).clamp(0.0, 1.0);
        match Self::SIGNAL {
            FeatureSignal::Positive => norm,
            FeatureSignal::Negative => 1.0 - norm,
        }
    }

    #[must_use]
    fn compute_feature_value(analysis: &PlacementAnalysis) -> BoardFeatureValue {
        let raw = Self::extract_raw(analysis);
        let transformed = Self::transform(raw);
        let normalized = Self::normalize(transformed);
        BoardFeatureValue {
            raw,
            transformed,
            normalized,
        }
    }
}

pub trait DynBoardFeatureSource: fmt::Debug + Send + Sync {
    #[must_use]
    fn id(&self) -> &'static str;
    #[must_use]
    fn name(&self) -> &'static str;
    #[must_use]
    fn type_name(&self) -> &'static str;
    #[must_use]
    fn normalization_min(&self) -> f32;
    #[must_use]
    fn normalization_max(&self) -> f32;
    #[must_use]
    fn signal(&self) -> FeatureSignal;
    #[must_use]
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32;
    #[must_use]
    fn transform(&self, raw: u32) -> f32;
    #[must_use]
    fn normalize(&self, transformed: f32) -> f32;
    #[must_use]
    fn compute_feature_value(&self, analysis: &PlacementAnalysis) -> BoardFeatureValue;
}

impl<T> DynBoardFeatureSource for T
where
    T: BoardFeatureSource,
{
    fn id(&self) -> &'static str {
        T::ID
    }

    fn name(&self) -> &'static str {
        T::NAME
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn normalization_min(&self) -> f32 {
        T::NORMALIZATION_MIN
    }

    fn normalization_max(&self) -> f32 {
        T::NORMALIZATION_MAX
    }

    fn signal(&self) -> FeatureSignal {
        T::SIGNAL
    }

    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        T::extract_raw(analysis)
    }

    fn transform(&self, raw: u32) -> f32 {
        T::transform(raw)
    }

    fn normalize(&self, transformed: f32) -> f32 {
        T::normalize(transformed)
    }

    fn compute_feature_value(&self, analysis: &PlacementAnalysis) -> BoardFeatureValue {
        T::compute_feature_value(analysis)
    }
}

/// Smooth penalty for covered holes (empty cells with blocks above).
///
/// This feature penalizes:
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
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (fewer holes is better).
#[derive(Debug)]
pub struct HolesPenalty;

impl BoardFeatureSource for HolesPenalty {
    const ID: &str = "holes_penalty";
    const NAME: &str = "Holes Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().num_holes().into()
    }
}

/// Smooth penalty for cumulative hole depth (weighted by blocks above each hole).
///
/// This feature penalizes:
///
/// - Deeply buried holes that are costly to clear
/// - Stacking that traps holes under tall columns
/// - Mid/late-game instability from unrecoverable cavities
///
/// Complements [`HolesPenalty`], which counts holes uniformly. Here, holes deeper in the stack contribute more
/// based on their depth (number of cells above them, including both occupied and empty cells).
///
/// # Raw measurement
///
/// - For each column, scan top-down, tracking a `depth` counter:
///   - When an occupied cell is encountered, increment `depth`.
///   - When an empty cell is encountered after at least one occupied cell (i.e., `depth > 0`):
///     - Add the current `depth` value to the cumulative sum.
///     - Increment `depth` (the hole itself adds to the depth for cells below).
/// - `raw = Σ (depth at each hole)` across all columns.
///
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (shallower holes is better).
#[derive(Debug)]
pub struct HoleDepthPenalty;

impl BoardFeatureSource for HoleDepthPenalty {
    const ID: &str = "hole_depth_penalty";
    const NAME: &str = "Hole Depth Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().sum_of_hole_depth()
    }
}

/// Smooth penalty for horizontal fragmentation by counting occupancy changes between adjacent cells within each row.
///
/// This feature penalizes:
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
/// By excluding walls, this feature evaluates edge and center placements fairly.
///
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (fewer transitions is better).
#[derive(Debug)]
pub struct RowTransitionsPenalty;

impl BoardFeatureSource for RowTransitionsPenalty {
    const ID: &str = "row_transitions_penalty";
    const NAME: &str = "Row Transitions Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().row_transitions()
    }
}

/// Smooth penalty for vertical fragmentation within columns by counting occupancy changes from top to bottom.
///
/// This feature penalizes:
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
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (fewer transitions is better).
#[derive(Debug)]
pub struct ColumnTransitionsPenalty;

impl BoardFeatureSource for ColumnTransitionsPenalty {
    const ID: &str = "column_transitions_penalty";
    const NAME: &str = "Column Transitions Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().column_transitions()
    }
}

/// Smooth penalty for surface height variation between adjacent columns.
///
/// This feature penalizes:
///
/// - Overall height differences between adjacent columns
/// - Step-like surface patterns
/// - Non-flat board surfaces that complicate piece placement
///
/// Differs from [`SurfaceRoughnessPenalty`] which measures curvature (second-order differences);
/// this feature directly measures first-order height differences, making it more sensitive
/// to simple step patterns and overall surface flatness.
///
/// # Raw measurement
///
/// - For each pair of adjacent columns, compute `|height_right - height_left|`.
/// - Sum across all adjacent pairs.
///
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (flatter surface is better).
#[derive(Debug)]
pub struct SurfaceBumpinessPenalty;

impl BoardFeatureSource for SurfaceBumpinessPenalty {
    const ID: &str = "surface_bumpiness_penalty";
    const NAME: &str = "Surface Bumpiness Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().surface_bumpiness()
    }
}

/// Smooth penalty for local surface curvature using second-order height differences (discrete Laplacian).
///
/// This feature penalizes:
///
/// - Small-scale surface unevenness
/// - Local height variations that increase future instability
/// - Shapes that are prone to creating holes
///
/// Differs from [`SurfaceBumpinessPenalty`] which measures first-order height differences;
/// this feature uses second-order differences (curvature) and is more sensitive to local
/// irregularities while tolerating gradual slopes. Complements row and column transitions
/// by remaining sensitive even when the overall stack is low.
///
/// # Raw measurement
///
/// - For each triplet of adjacent columns, compute the discrete Laplacian: `|(right - mid) - (mid - left)|`.
/// - Sum across all triplets.
///
/// # Normalization
///
/// - Clipped to `[P05, P95]`
/// - `SIGNAL` = Negative (flatter surface is better).
#[derive(Debug)]
pub struct SurfaceRoughnessPenalty;

impl BoardFeatureSource for SurfaceRoughnessPenalty {
    const ID: &str = "surface_roughness_penalty";
    const NAME: &str = "Surface Roughness Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().surface_roughness()
    }
}

/// Smooth penalty for excessive single-column well depth; thresholds shallow wells.
///
/// This feature penalizes:
///
/// - Over-committed vertical wells
/// - Single columns with extreme depth
/// - Over-commitment that reduces recovery options
///
/// Only wells deeper than 1 are considered dangerous. Shallow wells (depth ≤ 1) are allowed to preserve freedom
/// for controlled I-well construction. This feature is strictly a safety penalty and does NOT reward I-wells;
/// combine with [`IWellReward`] for balanced evaluation.
///
/// # Raw measurement
///
/// - `raw = Σ (depth - 1)` across all columns where `depth > 1`.
/// - Linear penalty for excess well depth beyond the threshold.
///
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (shallower wells is better).
#[derive(Debug)]
pub struct WellDepthPenalty;

impl BoardFeatureSource for WellDepthPenalty {
    const ID: &str = "well_depth_penalty";
    const NAME: &str = "Well Depth Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().sum_of_deep_well_depth()
    }
}

/// Thresholded risk for dangerously deep wells beyond safe operational limits.
///
/// This feature penalizes:
///
/// - Wells that exceed safe construction depth
/// - Over-commitment to vertical structures
/// - Reduced board flexibility and recovery options
///
/// Unlike [`WellDepthPenalty`], which applies smooth linear penalties to all wells beyond depth 1,
/// this feature focuses on extreme depth scenarios. Acts as a hard constraint to prevent catastrophic
/// well over-commitment while allowing controlled I-well construction.
///
/// # Raw measurement
///
/// - `raw = Σ (depth - DEPTH_THRESHOLD)` across columns where `depth > DEPTH_THRESHOLD` (2).
/// - Measures cumulative excess depth beyond the safe construction threshold.
///
/// # Normalization
///
/// - Clipped to `[P75, P95]`.
/// - `SIGNAL` = Negative (shallower wells is better).
#[derive(Debug)]
pub struct DeepWellRisk;

impl BoardFeatureSource for DeepWellRisk {
    const ID: &str = "deep_well_risk";
    const NAME: &str = "Deep Well Risk";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P75;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().sum_of_deep_well_depth()
    }
}

/// Smooth penalty for maximum column height across the board.
///
/// This feature penalizes:
///
/// - The tallest column on the board
/// - Localized vertical pressure
/// - Risk of reduced placement options
///
/// Unlike [`TopOutRisk`], which uses a thresholded approach focused on imminent danger,
/// this feature provides a smooth, continuous penalty throughout the game. It complements
/// [`TotalHeightPenalty`] by focusing on peak height rather than cumulative pressure.
///
/// # Raw measurement
///
/// - `raw = max(column_heights)`: the height of the tallest column.
///
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (lower maximum height is better).
#[derive(Debug)]
pub struct MaxHeightPenalty;

impl BoardFeatureSource for MaxHeightPenalty {
    const ID: &str = "max_height_penalty";
    const NAME: &str = "Max Height Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().max_height().into()
    }
}

/// Smooth penalty for maximum height in the center four columns.
///
/// This feature penalizes:
///
/// - Building high stacks in the center of the board
/// - Reduced flexibility for piece placement
/// - Difficulty in maintaining edge wells for tetrises
///
/// The center columns (indices 3-6 in the 10-column playable area) are strategically
/// important as they limit placement options more severely than edge columns.
/// High center stacks make it harder to build and maintain effective I-wells.
///
/// # Raw measurement
///
/// - `raw = max(column_heights[3..=6])`: maximum height among columns 3, 4, 5, and 6.
///
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (lower center height is better).
#[derive(Debug)]
pub struct CenterColumnsPenalty;

impl BoardFeatureSource for CenterColumnsPenalty {
    const ID: &str = "center_columns_penalty";
    const NAME: &str = "Center Columns Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().center_column_max_height().into()
    }
}

/// Thresholded risk for top-out in the center 4 columns (columns 3-6).
///
/// This feature penalizes:
///
/// - High stacking in the critical center columns
/// - Early-stage top-out risk in the most important area
/// - Center column height before it reaches critical levels
///
/// The center 4 columns are strategically crucial because:
/// - They are the most difficult to clear when filled
/// - Most pieces naturally gravitate toward the center
/// - High center columns severely restrict piece placement options
///
/// This feature uses the same thresholded approach as [`TopOutRisk`] (P75-P95), but focuses
/// specifically on the center 4 columns rather than all columns, allowing the AI to distinguish
/// between edge height and critical center height issues.
///
/// # Raw measurement
///
/// - `raw = max(column_heights[3..=6])`: the tallest column among the center 4 columns.
///
/// # Normalization
///
/// - Clipped to `[P75, P95]`.
/// - `SIGNAL` = Negative (lower height is better).
#[derive(Debug)]
pub struct CenterTopOutRisk;

impl BoardFeatureSource for CenterTopOutRisk {
    const ID: &str = "center_top_out_risk";
    const NAME: &str = "Center Top-Out Risk";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P75;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().center_column_max_height().into()
    }
}

/// Thresholded risk for imminent top-out based on maximum column height.
///
/// This feature penalizes:
///
/// - Approaching the ceiling (irreversible top-out risk)
/// - States close to game over
///
/// Unlike other features, max height is intentionally ignored for most of the game and only penalized near the ceiling,
/// reflecting the irreversible nature of top-out. Acts as a hard constraint rather than a general board quality measure.
///
/// # Raw measurement
///
/// - `raw = max(column_heights)`: the tallest column on the board.
///
/// # Normalization
///
/// - Clipped to `[P75, P95]`.
/// - `SIGNAL` = Negative (lower height is better).
#[derive(Debug)]
pub struct TopOutRisk;

impl BoardFeatureSource for TopOutRisk {
    const ID: &str = "top_out_risk";
    const NAME: &str = "Top-Out Risk";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P75;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().max_height().into()
    }
}

/// Smooth penalty for global stacking pressure by summing all column heights.
///
/// This feature penalizes:
///
/// - Gradual accumulation of blocks across the entire board
/// - Overall board pressure not captured by local roughness or transitions
/// - High average stack height
///
/// Unlike [`TopOutRisk`], which focuses on top-out danger from the tallest column, this feature captures
/// cumulative pressure across all columns. It reflects the total "weight" of the board state.
///
/// # Raw measurement
///
/// - `raw = Σ (column_heights)` across all 10 columns.
/// - Linear accumulation for a smooth, continuous penalty.
///
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (lower total height is better).
#[derive(Debug)]
pub struct TotalHeightPenalty;

impl BoardFeatureSource for TotalHeightPenalty {
    const ID: &str = "total_height_penalty";
    const NAME: &str = "Total Height Penalty";

    const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;
    const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;
    const SIGNAL: FeatureSignal = FeatureSignal::Negative;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().total_height().into()
    }
}

/// Discrete bonus for line clears with strong emphasis on efficient 4-line clears (tetrises).
///
/// This feature encourages:
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
/// - Discrete mapping: `[0→0.0, 1→0.0, 2→1.0, 3→2.0, 4→6.0]` (bonus weights)
/// - Singles (1 line) are not rewarded to discourage inefficient clearing.
/// - Tetrises receive 6× the reward of doubles, reflecting strategic importance.
///
/// # Stats
///
/// This is a per-placement reward, not a cumulative board state feature. Distribution depends on
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
/// - Clipped to `[0.0, 6.0]` (transformed range).
/// - `SIGNAL` = Positive (more lines cleared is better).
#[derive(Debug)]
pub struct LineClearBonus;

impl BoardFeatureSource for LineClearBonus {
    const ID: &str = "line_clear_bonus";
    const NAME: &str = "Line Clear Bonus";

    const NORMALIZATION_MIN: f32 = 0.0;
    const NORMALIZATION_MAX: f32 = 6.0;
    const SIGNAL: FeatureSignal = FeatureSignal::Positive;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        u32::try_from(analysis.cleared_lines()).unwrap()
    }

    fn transform(raw: u32) -> f32 {
        const WEIGHT: [f32; 5] = [0.0, 0.0, 1.0, 2.0, 6.0];
        WEIGHT[usize::try_from(raw).unwrap()]
    }
}

/// Smooth reward for maintaining an edge I-well for reliable tetrises without over-committing.
///
/// This feature encourages:
///
/// - Building a single-column well at the board edge
/// - Maintaining tetris-ready depth (around 4)
/// - Immediate consumption when I-piece is available
///
/// Considers only the leftmost and rightmost columns; center wells are ignored.
///
/// # Raw measurement
///
/// - `raw = max(left_well_depth, right_well_depth)`: the deeper of the two edge wells.
///
/// # Transform
///
/// - Triangular peak centered at depth 4 with width 2: `f(d) = 1 - |(d - 4) / (4/2)|`, clamped to $[0, 1]$.
/// - Smooth reward peaked at tetris-ready depth; decays linearly for wells that are too shallow or too deep.
///
/// # Normalization
///
/// - Clipped to `[NORMALIZATION_MIN=0.0, NORMALIZATION_MAX=1.0]` (transformed range already within bounds).
/// - `SIGNAL` = Positive (higher is better).
///
/// # Rationale and interplay
///
/// - Complements [`DeepWellRisk`] by focusing on edge wells suitable for tetrises, while [`DeepWellRisk`] penalizes excessive depths.
/// - Synergizes with [`LineClearBonus`] to favor consistent tetrises.
/// - The triangular transform naturally discourages both shallow wells (not ready) and overly deep wells (risky).
#[derive(Debug)]
pub struct IWellReward;

impl BoardFeatureSource for IWellReward {
    const ID: &str = "i_well_reward";
    const NAME: &str = "I-Well Reward";

    const NORMALIZATION_MIN: f32 = 0.0;
    const NORMALIZATION_MAX: f32 = 1.0;
    const SIGNAL: FeatureSignal = FeatureSignal::Positive;

    fn extract_raw(analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().edge_iwell_depth().into()
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(raw: u32) -> f32 {
        const PEAK: f32 = 4.0;
        const WIDTH: f32 = 2.0;
        let raw = raw as f32;
        (1.0 - ((raw - PEAK) / (PEAK / WIDTH)).abs()).clamp(0.0, 1.0)
    }
}
