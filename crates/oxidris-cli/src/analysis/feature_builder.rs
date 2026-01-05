//! Runtime feature construction from normalization parameters
//!
//! This module provides [`FeatureBuilder`] for constructing board features
//! with normalization parameters computed from actual gameplay data.
//!
//! # Overview
//!
//! Instead of using hardcoded percentiles, this builder computes normalization
//! parameters from session data and constructs features dynamically at runtime.
//!
//! # Feature Terminology
//!
//! Features follow a naming convention based on their behavior:
//!
//! - **Penalty**: Smooth negative signals across the full range (e.g., `num_holes_linear_penalty`)
//!   - Uses P05-P95 normalization for continuous feedback throughout the game
//!   - Applied to general board quality metrics without critical thresholds
//!
//! - **Risk**: Thresholded negative signals for dangerous states (e.g., `max_height_linear_risk`)
//!   - Uses P75-P95 normalization to focus on high-risk scenarios
//!   - Ignores safe values (below P75), only penalizes approaching danger zones
//!   - Reflects situations where the metric is only concerning at extreme values
//!
//! - **Reward**: Smooth positive signals (e.g., `i_well_reward`)
//!   - Encourages beneficial board configurations
//!   - Uses custom transformation (e.g., triangular peak for optimal I-well depth)
//!
//! - **Bonus**: Discrete strong rewards (e.g., `line_clear_bonus`)
//!   - Applied to per-placement actions (not cumulative board state)
//!   - Uses discrete mapping with emphasis on high-value actions (4-line tetrises)
//!
//! # Design Decisions
//!
//! ## Percentile-Based Normalization
//!
//! Features use percentile-based normalization for several reasons:
//!
//! - **Data-driven**: Grounded in actual gameplay behavior rather than arbitrary constants
//! - **Robust to outliers**: P05-P95 range clips extremes while preserving typical range
//! - **Simple and fast**: Linear scaling requires no complex computation at runtime
//!
//! ## Current Limitations
//!
//! ### Linear Transformation
//!
//! Most features use linear transformation (`raw as f32`), which doesn't capture non-linear
//! relationships between feature values and game outcomes. For example:
//!
//! - The first hole has much greater impact on survival than the 11th hole
//! - Linear transformation treats `holes: 0→1` the same as `holes: 10→11`
//! - This limitation affects all survival features (holes, height, etc.)
//!
//! ### Feature Redundancy
//!
//! The feature set has two types of redundancy:
//!
//! **1. Duplicate features with different normalization ranges:**
//!
//! - `max_height_linear_penalty` (P05-P95) vs `max_height_linear_risk` (P75-P95)
//! - `center_column_max_height_linear_penalty` vs `center_column_max_height_linear_risk`
//! - `sum_of_well_depth_linear_penalty` vs `sum_of_well_depth_linear_risk`
//!
//! These exist as an ad-hoc workaround to approximate non-linearity through different
//! scaling ranges. A systematic approach using non-linear transformations would be better.
//!
//! **2. Similar features measuring overlapping properties:**
//!
//! - Holes: count (`num_holes`) vs depth-weighted (`sum_of_hole_depth`)
//! - Surface: first-order (`surface_bumpiness`) vs second-order (`surface_roughness`)
//! - Transitions: horizontal (`row_transitions`) vs vertical (`column_transitions`)
//!
//! This creates issues:
//! - **Training difficulty**: Genetic algorithm must learn weights for correlated features
//! - **Interpretability**: Hard to determine which features are truly important
//! - **Computational cost**: Extra feature computation and evaluation overhead
//!
//! # Example
//!
//! ```no_run
//! use oxidris_cli::analysis::{RawBoardSample, RawFeatureStatistics};
//! use oxidris_cli::analysis::{BoardFeatureNormalizationParamCollection, FeatureBuilder};
//! use oxidris_evaluator::board_feature;
//! # let sessions = todo!();
//!
//! // 1. Get feature sources
//! let sources = board_feature::all_board_feature_sources();
//!
//! // 2. Extract raw samples from sessions
//! let raw_samples = RawBoardSample::from_sessions(&sources, &sessions);
//!
//! // 3. Compute statistics
//! let raw_stats = RawFeatureStatistics::from_samples(&sources, &raw_samples);
//!
//! // 4. Build normalization parameters
//! let norm_params = BoardFeatureNormalizationParamCollection::from_stats(&sources, &raw_stats);
//!
//! // 5. Construct features with runtime normalization
//! let builder = FeatureBuilder::new(norm_params);
//! let features = builder.build_all_features().unwrap();
//! ```

use anyhow::Context as _;
use oxidris_evaluator::board_feature::{
    BoardFeatureSource, BoxedBoardFeature, FeatureSignal, IWellReward, LineClearBonus,
    LinearNormalized,
    source::{
        CenterColumnMaxHeight, ColumnTransitions, EdgeIWellDepth, MaxHeight, NumClearedLines,
        NumHoles, RowTransitions, SumOfHoleDepth, SumOfWellDepth, SurfaceBumpiness,
        SurfaceRoughness, TotalHeight,
    },
};

use crate::analysis::{BoardFeatureNormalizationParam, BoardFeatureNormalizationParamCollection};

/// Feature builder that constructs features from runtime parameters
pub struct FeatureBuilder {
    params: BoardFeatureNormalizationParamCollection,
}

impl FeatureBuilder {
    pub fn new(params: BoardFeatureNormalizationParamCollection) -> Self {
        Self { params }
    }

    /// Build all board features with runtime normalization
    pub fn build_all_features(&self) -> anyhow::Result<Vec<BoxedBoardFeature>> {
        Ok(vec![
            // Survival features
            self.build_num_holes_linear_penalty()?,
            self.build_sum_of_hole_depth_linear_penalty()?,
            self.build_max_height_linear_penalty()?,
            self.build_max_height_linear_risk()?,
            self.build_center_column_max_height_linear_penalty()?,
            self.build_center_column_max_height_linear_risk()?,
            self.build_total_height_linear_penalty()?,
            // Structure features
            self.build_surface_bumpiness_linear_penalty()?,
            self.build_surface_roughness_linear_penalty()?,
            self.build_row_transitions_linear_penalty()?,
            self.build_column_transitions_linear_penalty()?,
            self.build_well_depth_linear_penalty()?,
            self.build_well_depth_linear_risk()?,
            // Score features
            self.build_line_clear_bonus(),
            self.build_i_well_reward(),
        ])
    }

    /// Build a linear penalty feature with P05-P95 normalization range.
    ///
    /// Linear penalties provide smooth, continuous negative signals throughout the game,
    /// using the full distribution (P05-P95) to normalize feature values.
    ///
    /// # Normalization strategy
    ///
    /// - **Range**: P05-P95 (captures most of the distribution)
    /// - **Signal**: Negative (lower raw values are better)
    /// - **Clipping**: Values outside [P05, P95] are clipped
    /// - **Mapping**: Linear interpolation from [P05, P95] → [0.0, 1.0]
    ///
    /// This approach provides gradual feedback across the entire game, making it suitable
    /// for general board quality metrics that don't have critical thresholds.
    fn build_linear_penalty_for<S>(&self, source: &S) -> anyhow::Result<BoxedBoardFeature>
    where
        S: BoardFeatureSource,
    {
        let param = self.get_param(source)?;
        Ok(Box::new(LinearNormalized::new(
            format!("{}_linear_penalty", source.id()).into(),
            format!("{} (Linear Penalty)", source.name()).into(),
            FeatureSignal::Negative,
            param.value_percentiles.p05,
            param.value_percentiles.p95,
            source.clone_boxed(),
        )))
    }

    /// Build a linear risk feature with P75-P95 normalization range (threshold-based).
    ///
    /// Linear risk features provide thresholded negative signals that focus on dangerous
    /// states, ignoring most of the game and only penalizing high-risk scenarios.
    ///
    /// # Normalization strategy
    ///
    /// - **Range**: P75-P95 (upper tail of distribution)
    /// - **Signal**: Negative (lower raw values are better)
    /// - **Clipping**: Values outside [P75, P95] are clipped
    /// - **Mapping**: Linear interpolation from [P75, P95] → [0.0, 1.0]
    ///
    /// This threshold-based approach reflects situations where the metric is only concerning
    /// at extreme values (e.g., top-out risk only matters near the ceiling). Values below P75
    /// are considered safe and map to 0.0.
    fn build_linear_risk_for<S>(&self, source: &S) -> anyhow::Result<BoxedBoardFeature>
    where
        S: BoardFeatureSource,
    {
        let param = self.get_param(source)?;
        Ok(Box::new(LinearNormalized::new(
            format!("{}_linear_risk", source.id()).into(),
            format!("{} (Linear Risk)", source.name()).into(),
            FeatureSignal::Negative,
            param.value_percentiles.p75,
            param.value_percentiles.p95,
            source.clone_boxed(),
        )))
    }

    /// Build smooth penalty for number of holes across the board.
    ///
    /// Penalizes trapped empty spaces (holes) that are difficult to fill without clearing lines.
    /// Uses P05-P95 normalization for continuous feedback throughout the game.
    ///
    /// # Feature ID
    ///
    /// `num_holes_linear_penalty`
    fn build_num_holes_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&NumHoles)
    }

    /// Build smooth penalty for cumulative hole depth (weighted by blocks above).
    ///
    /// Penalizes deeply buried holes that are costly to clear. Unlike [`build_num_holes_linear_penalty`],
    /// this weights holes by their depth, making deeply buried holes contribute more to the penalty.
    /// Uses P05-P95 normalization for continuous feedback.
    ///
    /// # Feature ID
    ///
    /// `sum_of_hole_depth_linear_penalty`
    fn build_sum_of_hole_depth_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&SumOfHoleDepth)
    }

    /// Build smooth penalty for maximum column height.
    ///
    /// Penalizes the tallest column throughout the game, providing continuous feedback on
    /// localized vertical pressure. Complements [`build_max_height_linear_risk`] which focuses
    /// on imminent top-out danger. Uses P05-P95 normalization.
    ///
    /// # Feature ID
    ///
    /// `max_height_linear_penalty`
    fn build_max_height_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&MaxHeight)
    }

    /// Build thresholded risk for imminent top-out based on maximum height.
    ///
    /// Focuses on dangerous states close to the ceiling. Unlike [`build_max_height_linear_penalty`],
    /// this ignores most of the game (below P75) and only penalizes approaching the ceiling,
    /// reflecting the irreversible nature of top-out. Uses P75-P95 normalization.
    ///
    /// # Feature ID
    ///
    /// `max_height_linear_risk`
    fn build_max_height_linear_risk(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_risk_for(&MaxHeight)
    }

    /// Build smooth penalty for center column height (columns 3-6).
    ///
    /// Penalizes high stacking in the strategically critical center 4 columns throughout the game.
    /// Center columns are harder to clear and restrict piece placement more than edge columns.
    /// Uses P05-P95 normalization.
    ///
    /// # Feature ID
    ///
    /// `center_column_max_height_linear_penalty`
    fn build_center_column_max_height_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&CenterColumnMaxHeight)
    }

    /// Build thresholded risk for center column top-out.
    ///
    /// Focuses on dangerous center column height that threatens early top-out. Uses the same
    /// thresholded approach as [`build_max_height_linear_risk`] but specifically for the center
    /// 4 columns. Uses P75-P95 normalization.
    ///
    /// # Feature ID
    ///
    /// `center_column_max_height_linear_risk`
    fn build_center_column_max_height_linear_risk(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_risk_for(&CenterColumnMaxHeight)
    }

    /// Build smooth penalty for total height (sum of all column heights).
    ///
    /// Penalizes global stacking pressure by measuring cumulative height across all columns.
    /// Captures overall board "weight" and complements localized height metrics. Uses P05-P95
    /// normalization for continuous feedback.
    ///
    /// # Feature ID
    ///
    /// `total_height_linear_penalty`
    fn build_total_height_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&TotalHeight)
    }

    /// Build smooth penalty for horizontal fragmentation (row transitions).
    ///
    /// Penalizes horizontally fragmented structures with narrow gaps and broken rows.
    /// Uses P05-P95 normalization for continuous feedback on structural quality.
    ///
    /// # Feature ID
    ///
    /// `row_transitions_linear_penalty`
    fn build_row_transitions_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&RowTransitions)
    }

    /// Build smooth penalty for vertical fragmentation (column transitions).
    ///
    /// Penalizes vertical fragmentation within columns, including internal splits and covered holes.
    /// Uses P05-P95 normalization for continuous feedback on vertical structure quality.
    ///
    /// # Feature ID
    ///
    /// `column_transitions_linear_penalty`
    fn build_column_transitions_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&ColumnTransitions)
    }

    /// Build smooth penalty for surface height variation (first-order differences).
    ///
    /// Penalizes step-like surface patterns and non-flat surfaces that complicate piece placement.
    /// Measures first-order height differences between adjacent columns. Uses P05-P95 normalization.
    ///
    /// # Feature ID
    ///
    /// `surface_bumpiness_linear_penalty`
    fn build_surface_bumpiness_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&SurfaceBumpiness)
    }

    /// Build smooth penalty for local surface curvature (second-order differences).
    ///
    /// Penalizes small-scale surface unevenness using discrete Laplacian (second-order differences).
    /// More sensitive to local irregularities than [`build_surface_bumpiness_linear_penalty`]
    /// while tolerating gradual slopes. Uses P05-P95 normalization.
    ///
    /// # Feature ID
    ///
    /// `surface_roughness_linear_penalty`
    fn build_surface_roughness_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&SurfaceRoughness)
    }

    /// Build smooth penalty for well depth beyond safe threshold.
    ///
    /// Penalizes over-committed vertical wells throughout the game. Only wells deeper than 1
    /// are considered (shallow wells are allowed for controlled play). Complements
    /// [`build_well_depth_linear_risk`] which focuses on extreme well depth. Uses P05-P95
    /// normalization.
    ///
    /// # Feature ID
    ///
    /// `sum_of_well_depth_linear_penalty`
    fn build_well_depth_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&SumOfWellDepth)
    }

    /// Build thresholded risk for dangerously deep wells.
    ///
    /// Focuses on extreme well depth that threatens board flexibility. Unlike
    /// [`build_well_depth_linear_penalty`], this ignores moderate wells and only penalizes
    /// wells beyond safe operational limits. Uses P75-P95 normalization.
    ///
    /// # Feature ID
    ///
    /// `sum_of_well_depth_linear_risk`
    fn build_well_depth_linear_risk(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_risk_for(&SumOfWellDepth)
    }

    /// Build discrete bonus for line clears with emphasis on 4-line tetrises.
    ///
    /// Rewards efficient line clearing with strong preference for 4-line clears (tetrises).
    /// This is a per-placement action reward, not a cumulative board state metric.
    ///
    /// See [`LineClearBonus`] for transform and normalization details.
    ///
    /// # Feature ID
    ///
    /// `line_clear_bonus`
    #[expect(clippy::unused_self)]
    fn build_line_clear_bonus(&self) -> BoxedBoardFeature {
        let source = NumClearedLines;
        Box::new(LineClearBonus::new(
            "line_clear_bonus".into(),
            "Line Clear Bonus".into(),
            source,
        ))
    }

    /// Build triangular reward for optimal I-piece well setup at board edges.
    ///
    /// Rewards I-piece well construction at optimal depth (~4) for efficient tetris execution.
    /// This balances the well depth penalties by recognizing when vertical wells are strategic.
    ///
    /// See [`IWellReward`] for transform and normalization details.
    ///
    /// # Feature ID
    ///
    /// `i_well_reward`
    #[expect(clippy::unused_self)]
    fn build_i_well_reward(&self) -> BoxedBoardFeature {
        let source = EdgeIWellDepth;
        Box::new(IWellReward::new(
            "i_well_reward".into(),
            "I-Well Reward".into(),
            source,
        ))
    }

    /// Get normalization parameters for a specific feature source
    fn get_param<S>(&self, source: &S) -> anyhow::Result<&BoardFeatureNormalizationParam>
    where
        S: BoardFeatureSource,
    {
        self.params
            .get(source)
            .with_context(|| format!("Missing percentiles for feature source '{}'", source.id()))
    }
}
