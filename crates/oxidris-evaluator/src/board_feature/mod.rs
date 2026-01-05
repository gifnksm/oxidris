//! Normalized evaluation features for Tetris board states.
//!
//! Provides board evaluation features for Tetris, each producing a normalized score in \[0.0, 1.0\].
//! Higher is always better after normalization; negative features are inverted via [`FeatureSignal::Negative`].
//!
//! # Typology
//!
//! Features follow a naming convention based on their behavior:
//!
//! - **Risk**: Thresholded danger that escalates rapidly beyond a safe limit (e.g., [`LINEAR_DEEP_WELL_RISK`], [`LINEAR_TOP_OUT_RISK`])
//! - **Penalty**: Smooth negative signals (e.g., [`LINEAR_HOLES_PENALTY`], [`LINEAR_HOLE_DEPTH_PENALTY`], [`LINEAR_ROW_TRANSITIONS_PENALTY`])
//! - **Reward**: Smooth positive signals (e.g., [`I_WELL_REWARD`])
//! - **Bonus**: Discrete strong rewards (e.g., [`LINE_CLEAR_BONUS`])
//!
//! # Feature Categories
//!
//! Features are categorized by their role in gameplay:
//!
//! ## Survival Features
//!
//! Directly affect game termination (when the game ends):
//!
//! - [`LINEAR_HOLES_PENALTY`] - Number of holes (empty cells with blocks above)
//! - [`LINEAR_HOLE_DEPTH_PENALTY`] - Sum of depths of all holes
//! - [`LINEAR_MAX_HEIGHT_PENALTY`] - Maximum column height
//! - [`LINEAR_TOTAL_HEIGHT_PENALTY`] - Sum of all column heights
//! - [`LINEAR_CENTER_COLUMNS_PENALTY`] - Sum of center column heights (columns 3-6)
//! - [`LINEAR_TOP_OUT_RISK`] - Risk of topping out (height-based threshold)
//! - [`LINEAR_CENTER_TOP_OUT_RISK`] - Risk of topping out in center columns
//!
//! ## Structure Features
//!
//! Affect placement flexibility and future options:
//!
//! - [`LINEAR_SURFACE_BUMPINESS_PENALTY`] - Sum of absolute height differences between adjacent columns
//! - [`LINEAR_SURFACE_ROUGHNESS_PENALTY`] - Variance in column heights
//! - [`LINEAR_ROW_TRANSITIONS_PENALTY`] - Number of horizontal empty-to-filled transitions
//! - [`LINEAR_COLUMN_TRANSITIONS_PENALTY`] - Number of vertical empty-to-filled transitions
//! - [`LINEAR_WELL_DEPTH_PENALTY`] - Depth of deepest well (reduces placement flexibility)
//! - [`LINEAR_DEEP_WELL_RISK`] - Risk from excessively deep wells (reduces recovery options)
//!
//! ## Score Features
//!
//! Directly contribute to game score:
//!
//! - [`LINE_CLEAR_BONUS`] - Number of lines cleared by this placement
//! - [`I_WELL_REWARD`] - Quality of I-piece well setup (depth ~4 is optimal)
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
//! - [`LINE_CLEAR_BONUS`]: Exponential weighting (4-line Tetris gets 6× weight)
//! - [`I_WELL_REWARD`]: Triangular peak centered at depth 4 (optimal for I-pieces)
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
//!    - [`LINEAR_TOP_OUT_RISK`] vs [`LINEAR_MAX_HEIGHT_PENALTY`] - Both measure maximum height but with different normalization
//!    - [`LINEAR_CENTER_TOP_OUT_RISK`] vs [`LINEAR_CENTER_COLUMNS_PENALTY`] - Both measure center column heights
//!    - [`LINEAR_DEEP_WELL_RISK`] vs [`LINEAR_WELL_DEPTH_PENALTY`] - Both measure well depth
//!
//!    These duplicates exist as an ad-hoc attempt to capture non-linearity through different
//!    scaling ranges, suggesting a systematic need for non-linear transformations.
//!
//! 2. **Similar features measuring overlapping properties**:
//!
//!    - [`NumHoles`] vs [`SumOfHoleDepth`] - Both measure holes (count vs depth)
//!    - [`SurfaceBumpiness`] vs [`SurfaceRoughness`] - Both measure surface irregularity
//!    - [`RowTransitions`] vs [`ColumnTransitions`] - Both measure board complexity
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
//! use oxidris_evaluator::board_feature;
//! use oxidris_evaluator::placement_evaluator::FeatureBasedPlacementEvaluator;
//!
//! // Create evaluator with all features
//! let features = board_feature::all_board_features();
//! let weights = vec![1.0; features.len()];
//! let evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
//! ```
//!
//! Individual features can be computed directly using feature constants:
//!
//! ```rust,no_run
//! use oxidris_evaluator::{placement_analysis::PlacementAnalysis, board_feature::{BoardFeature, LINEAR_HOLES_PENALTY}};
//!
//! let analysis: PlacementAnalysis = todo!();
//!
//! // Compute full feature value (raw → transform → normalize)
//! let feature_value = LINEAR_HOLES_PENALTY.compute_feature_value(&analysis);
//!
//! // Or access individual steps
//! let raw = LINEAR_HOLES_PENALTY.extract_raw(&analysis);
//! let transformed = LINEAR_HOLES_PENALTY.transform(raw);
//! let normalized = LINEAR_HOLES_PENALTY.normalize(transformed);
//! ```
//!
//! # Trait Architecture
//!
//! The feature system uses two main traits:
//!
//! ## [`BoardFeature`] - Public API
//!
//! The main trait for feature objects, providing:
//!
//! - Metadata access: `id()`, `name()`
//! - Feature computation: `extract_raw()`, `transform()`, `normalize()`
//! - Complete pipeline: `compute_feature_value()`
//! - Dynamic dispatch: `clone_boxed()`
//!
//! All feature constants (e.g., [`LINEAR_HOLES_PENALTY`]) implement this trait.
//!
//! ## [`BoardFeatureSource`] - Internal Implementation
//!
//! A minimal trait for extracting raw values from board states:
//!
//! - Feature computation: `extract_raw()` method only
//!
//! Feature source types (e.g., [`NumHoles`]) implement this trait and are wrapped
//! by [`LinearNormalized`] to provide transformation and normalization.
//!
//! [`all_board_features()`] provides access to all active features for batch processing.

use std::{borrow::Cow, fmt};

use crate::board_feature::source::{EdgeIWellDepth, NumClearedLines};

use crate::{
    board_feature::source::{
        CenterColumnMaxHeight, ColumnTransitions, MaxHeight, NumHoles, RowTransitions,
        SumOfHoleDepth, SumOfWellDepth, SurfaceBumpiness, SurfaceRoughness, TotalHeight,
    },
    placement_analysis::PlacementAnalysis,
};

pub mod source;
mod stats;

#[must_use]
pub fn all_board_features() -> Vec<BoxedBoardFeature> {
    vec![
        // survival features
        Box::new(LINEAR_HOLES_PENALTY),
        Box::new(LINEAR_HOLE_DEPTH_PENALTY),
        Box::new(LINEAR_MAX_HEIGHT_PENALTY),
        Box::new(LINEAR_CENTER_COLUMNS_PENALTY),
        Box::new(LINEAR_TOTAL_HEIGHT_PENALTY),
        Box::new(LINEAR_TOP_OUT_RISK),
        Box::new(LINEAR_CENTER_TOP_OUT_RISK),
        // structure features
        Box::new(LINEAR_SURFACE_BUMPINESS_PENALTY),
        Box::new(LINEAR_SURFACE_ROUGHNESS_PENALTY),
        Box::new(LINEAR_ROW_TRANSITIONS_PENALTY),
        Box::new(LINEAR_COLUMN_TRANSITIONS_PENALTY),
        Box::new(LINEAR_WELL_DEPTH_PENALTY),
        Box::new(LINEAR_DEEP_WELL_RISK),
        // score features
        Box::new(LINE_CLEAR_BONUS),
        Box::new(I_WELL_REWARD),
    ]
}

#[must_use]
pub fn all_board_feature_sources() -> Vec<BoxedBoardFeatureSource> {
    vec![
        // survival features
        Box::new(NumHoles),
        Box::new(SumOfHoleDepth),
        Box::new(MaxHeight),
        Box::new(CenterColumnMaxHeight),
        Box::new(TotalHeight),
        // structure features
        Box::new(SurfaceBumpiness),
        Box::new(SurfaceRoughness),
        Box::new(RowTransitions),
        Box::new(ColumnTransitions),
        Box::new(SumOfWellDepth),
        // score features
        Box::new(NumClearedLines),
        Box::new(EdgeIWellDepth),
    ]
}

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

pub trait BoardFeatureSource {
    #[must_use]
    fn id(&self) -> &str;
    #[must_use]
    fn name(&self) -> &str;
    #[must_use]
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
    #[must_use]
    fn clone_boxed(&self) -> BoxedBoardFeatureSource;
    #[must_use]
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32;
}

pub type BoxedBoardFeatureSource = Box<dyn BoardFeatureSource>;

impl Clone for BoxedBoardFeatureSource {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

fn linear_normalize(val: f32, signal: FeatureSignal, min: f32, max: f32) -> f32 {
    let span = max - min;
    let norm = ((val - min) / span).clamp(0.0, 1.0);
    match signal {
        FeatureSignal::Positive => norm,
        FeatureSignal::Negative => 1.0 - norm,
    }
}

pub trait BoardFeature: fmt::Debug + Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn feature_source(&self) -> &dyn BoardFeatureSource;
    fn clone_boxed(&self) -> BoxedBoardFeature;

    #[must_use]
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32;

    #[must_use]
    fn transform(&self, raw: u32) -> f32;

    #[must_use]
    fn normalize(&self, transformed: f32) -> f32;

    #[must_use]
    fn compute_feature_value(&self, analysis: &PlacementAnalysis) -> BoardFeatureValue {
        let raw = self.extract_raw(analysis);
        let transformed = self.transform(raw);
        let normalized = self.normalize(transformed);
        BoardFeatureValue {
            raw,
            transformed,
            normalized,
        }
    }
}

pub type BoxedBoardFeature = Box<dyn BoardFeature>;

impl Clone for BoxedBoardFeature {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

#[derive(Debug, Clone)]
pub struct LinearNormalized<S> {
    id: Cow<'static, str>,
    name: Cow<'static, str>,
    signal: FeatureSignal,
    normalize_min: f32,
    normalize_max: f32,
    source: S,
}

impl<S> LinearNormalized<S> {
    pub const fn new(
        id: Cow<'static, str>,
        name: Cow<'static, str>,
        signal: FeatureSignal,
        normalize_min: f32,
        normalize_max: f32,
        source: S,
    ) -> Self {
        Self {
            id,
            name,
            signal,
            normalize_min,
            normalize_max,
            source,
        }
    }
}

impl<S> BoardFeature for LinearNormalized<S>
where
    S: BoardFeatureSource + Clone + fmt::Debug + Send + Sync + 'static,
{
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn feature_source(&self) -> &dyn BoardFeatureSource {
        &self.source
    }

    fn clone_boxed(&self) -> BoxedBoardFeature {
        Box::new(self.clone())
    }

    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        self.source.extract_raw(analysis)
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(&self, raw: u32) -> f32 {
        raw as f32
    }

    fn normalize(&self, transformed: f32) -> f32 {
        linear_normalize(
            transformed,
            self.signal,
            self.normalize_min,
            self.normalize_max,
        )
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
pub const LINEAR_HOLES_PENALTY: LinearNormalized<NumHoles> = LinearNormalized::new(
    Cow::Borrowed("linear_holes_penalty"),
    Cow::Borrowed("Number of Holes Penalty (Linear)"),
    FeatureSignal::Negative,
    NumHoles::P05,
    NumHoles::P95,
    NumHoles,
);

/// Smooth penalty for cumulative hole depth (weighted by blocks above each hole).
///
/// This feature penalizes:
///
/// - Deeply buried holes that are costly to clear
/// - Stacking that traps holes under tall columns
/// - Mid/late-game instability from unrecoverable cavities
///
/// Complements [`LINEAR_HOLES_PENALTY`], which counts holes uniformly. Here, holes deeper in the stack contribute more
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
pub const LINEAR_HOLE_DEPTH_PENALTY: LinearNormalized<SumOfHoleDepth> = LinearNormalized::new(
    Cow::Borrowed("linear_hole_depth_penalty"),
    Cow::Borrowed("Sum of Hole Depth Penalty (Linear)"),
    FeatureSignal::Negative,
    SumOfHoleDepth::P05,
    SumOfHoleDepth::P95,
    SumOfHoleDepth,
);

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
pub const LINEAR_ROW_TRANSITIONS_PENALTY: LinearNormalized<RowTransitions> = LinearNormalized::new(
    Cow::Borrowed("linear_row_transitions_penalty"),
    Cow::Borrowed("Row Transitions Penalty (Linear)"),
    FeatureSignal::Negative,
    RowTransitions::P05,
    RowTransitions::P95,
    RowTransitions,
);

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
pub const LINEAR_COLUMN_TRANSITIONS_PENALTY: LinearNormalized<ColumnTransitions> =
    LinearNormalized::new(
        Cow::Borrowed("linear_column_transitions_penalty"),
        Cow::Borrowed("Column Transitions Penalty (Linear)"),
        FeatureSignal::Negative,
        ColumnTransitions::P05,
        ColumnTransitions::P95,
        ColumnTransitions,
    );

/// Smooth penalty for surface height variation between adjacent columns.
///
/// This feature penalizes:
///
/// - Overall height differences between adjacent columns
/// - Step-like surface patterns
/// - Non-flat board surfaces that complicate piece placement
///
/// Differs from [`LINEAR_SURFACE_ROUGHNESS_PENALTY`] which measures curvature (second-order differences);
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
pub const LINEAR_SURFACE_BUMPINESS_PENALTY: LinearNormalized<SurfaceBumpiness> =
    LinearNormalized::new(
        Cow::Borrowed("linear_surface_bumpiness_penalty"),
        Cow::Borrowed("Surface Bumpiness Penalty (Linear)"),
        FeatureSignal::Negative,
        SurfaceBumpiness::P05,
        SurfaceBumpiness::P95,
        SurfaceBumpiness,
    );

/// Smooth penalty for local surface curvature using second-order height differences (discrete Laplacian).
///
/// This feature penalizes:
///
/// - Small-scale surface unevenness
/// - Local height variations that increase future instability
/// - Shapes that are prone to creating holes
///
/// Differs from [`LINEAR_SURFACE_BUMPINESS_PENALTY`] which measures first-order height differences;
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
pub const LINEAR_SURFACE_ROUGHNESS_PENALTY: LinearNormalized<SurfaceRoughness> =
    LinearNormalized::new(
        Cow::Borrowed("linear_surface_roughness_penalty"),
        Cow::Borrowed("Surface Roughness Penalty (Linear)"),
        FeatureSignal::Negative,
        SurfaceRoughness::P05,
        SurfaceRoughness::P95,
        SurfaceRoughness,
    );

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
/// combine with [`I_WELL_REWARD`] for balanced evaluation.
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
pub const LINEAR_WELL_DEPTH_PENALTY: LinearNormalized<SumOfWellDepth> = LinearNormalized::new(
    Cow::Borrowed("linear_well_depth_penalty"),
    Cow::Borrowed("Well Depth Penalty (Linear)"),
    FeatureSignal::Negative,
    SumOfWellDepth::P05,
    SumOfWellDepth::P95,
    SumOfWellDepth,
);

/// Thresholded risk for dangerously deep wells beyond safe operational limits.
///
/// This feature penalizes:
///
/// - Wells that exceed safe construction depth
/// - Over-commitment to vertical structures
/// - Reduced board flexibility and recovery options
///
/// Unlike [`LINEAR_WELL_DEPTH_PENALTY`], which applies smooth linear penalties to all wells beyond depth 1,
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
pub const LINEAR_DEEP_WELL_RISK: LinearNormalized<SumOfWellDepth> = LinearNormalized::new(
    Cow::Borrowed("linear_deep_well_risk"),
    Cow::Borrowed("Deep Well Risk (Linear)"),
    FeatureSignal::Negative,
    SumOfWellDepth::P75,
    SumOfWellDepth::P95,
    SumOfWellDepth,
);

/// Smooth penalty for maximum column height across the board.
///
/// This feature penalizes:
///
/// - The tallest column on the board
/// - Localized vertical pressure
/// - Risk of reduced placement options
///
/// Unlike [`LINEAR_TOP_OUT_RISK`], which uses a thresholded approach focused on imminent danger,
/// this feature provides a smooth, continuous penalty throughout the game. It complements
/// [`LINEAR_TOTAL_HEIGHT_PENALTY`] by focusing on peak height rather than cumulative pressure.
///
/// # Raw measurement
///
/// - `raw = max(column_heights)`: the height of the tallest column.
///
/// # Normalization
///
/// - Clipped to `[P05, P95]`.
/// - `SIGNAL` = Negative (lower maximum height is better).
pub const LINEAR_MAX_HEIGHT_PENALTY: LinearNormalized<MaxHeight> = LinearNormalized::new(
    Cow::Borrowed("linear_max_height_penalty"),
    Cow::Borrowed("Max Height Penalty (Linear)"),
    FeatureSignal::Negative,
    MaxHeight::P05,
    MaxHeight::P95,
    MaxHeight,
);

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
pub const LINEAR_TOP_OUT_RISK: LinearNormalized<MaxHeight> = LinearNormalized::new(
    Cow::Borrowed("linear_top_out_risk"),
    Cow::Borrowed("Top-Out Risk (Linear)"),
    FeatureSignal::Negative,
    MaxHeight::P75,
    MaxHeight::P95,
    MaxHeight,
);

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
pub const LINEAR_CENTER_COLUMNS_PENALTY: LinearNormalized<CenterColumnMaxHeight> =
    LinearNormalized::new(
        Cow::Borrowed("linear_center_columns_penalty"),
        Cow::Borrowed("Center Columns Max Height Penalty (Linear)"),
        FeatureSignal::Negative,
        CenterColumnMaxHeight::P05,
        CenterColumnMaxHeight::P95,
        CenterColumnMaxHeight,
    );

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
/// This feature uses the same thresholded approach as [`LINEAR_TOP_OUT_RISK`] (P75-P95), but focuses
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
pub const LINEAR_CENTER_TOP_OUT_RISK: LinearNormalized<CenterColumnMaxHeight> =
    LinearNormalized::new(
        Cow::Borrowed("linear_center_top_out_risk"),
        Cow::Borrowed("Center Top-Out Risk (Linear)"),
        FeatureSignal::Negative,
        CenterColumnMaxHeight::P75,
        CenterColumnMaxHeight::P95,
        CenterColumnMaxHeight,
    );

/// Smooth penalty for global stacking pressure by summing all column heights.
///
/// This feature penalizes:
///
/// - Gradual accumulation of blocks across the entire board
/// - Overall board pressure not captured by local roughness or transitions
/// - High average stack height
///
/// Unlike [`LINEAR_TOP_OUT_RISK`], which focuses on top-out danger from the tallest column, this feature captures
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
pub const LINEAR_TOTAL_HEIGHT_PENALTY: LinearNormalized<TotalHeight> = LinearNormalized::new(
    Cow::Borrowed("linear_total_height_penalty"),
    Cow::Borrowed("Total Height Penalty (Linear)"),
    FeatureSignal::Negative,
    TotalHeight::P05,
    TotalHeight::P95,
    TotalHeight,
);

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
pub const LINE_CLEAR_BONUS: LineClearBonus = LineClearBonus::new(NumClearedLines);

#[derive(Debug, Clone)]
pub struct LineClearBonus {
    source: NumClearedLines,
}

impl LineClearBonus {
    #[must_use]
    pub const fn new(source: NumClearedLines) -> Self {
        Self { source }
    }
}

impl BoardFeature for LineClearBonus {
    fn id(&self) -> &'static str {
        "line_clear_bonus"
    }

    fn name(&self) -> &'static str {
        "Line Clear Bonus"
    }

    fn feature_source(&self) -> &dyn BoardFeatureSource {
        &self.source
    }

    fn clone_boxed(&self) -> BoxedBoardFeature {
        Box::new(self.clone())
    }

    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        self.source.extract_raw(analysis)
    }

    fn transform(&self, raw: u32) -> f32 {
        const WEIGHT: [f32; 5] = [0.0, 0.0, 1.0, 2.0, 6.0];
        WEIGHT[usize::try_from(raw).unwrap()]
    }

    fn normalize(&self, transformed: f32) -> f32 {
        linear_normalize(transformed, FeatureSignal::Positive, 0.0, 6.0)
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
/// - Complements [`LINEAR_DEEP_WELL_RISK`] by focusing on edge wells suitable for tetrises, while [`LINEAR_DEEP_WELL_RISK`] penalizes excessive depths.
/// - Synergizes with [`LINE_CLEAR_BONUS`] to favor consistent tetrises.
/// - The triangular transform naturally discourages both shallow wells (not ready) and overly deep wells (risky).
pub const I_WELL_REWARD: IWellReward = IWellReward::new(EdgeIWellDepth);

#[derive(Debug, Clone)]
pub struct IWellReward {
    source: EdgeIWellDepth,
}

impl IWellReward {
    #[must_use]
    pub const fn new(source: EdgeIWellDepth) -> Self {
        Self { source }
    }
}

impl BoardFeature for IWellReward {
    fn id(&self) -> &'static str {
        "i_well_reward"
    }

    fn name(&self) -> &'static str {
        "I-Well Reward"
    }

    fn feature_source(&self) -> &dyn BoardFeatureSource {
        &self.source
    }

    fn clone_boxed(&self) -> BoxedBoardFeature {
        Box::new(self.clone())
    }

    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().edge_i_well_depth().into()
    }

    #[expect(clippy::cast_precision_loss)]
    fn transform(&self, raw: u32) -> f32 {
        const PEAK: f32 = 4.0;
        const WIDTH: f32 = 2.0;
        let raw = raw as f32;
        (1.0 - ((raw - PEAK) / (PEAK / WIDTH)).abs()).clamp(0.0, 1.0)
    }

    fn normalize(&self, transformed: f32) -> f32 {
        linear_normalize(transformed, FeatureSignal::Positive, 0.0, 1.0)
    }
}
