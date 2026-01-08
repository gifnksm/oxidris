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
//! - **Penalty**: Smooth negative signals across the full range (e.g., `num_holes_raw_penalty`)
//!   - Uses P05-P95 normalization for continuous feedback throughout the game
//!   - Applied to general board quality metrics without critical thresholds
//!
//! - **Risk**: Thresholded negative signals for dangerous states (e.g., `max_height_raw_risk`)
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
//! - `max_height_raw_penalty` (P05-P95) vs `max_height_raw_risk` (P75-P95)
//! - `center_column_max_height_raw_penalty` vs `center_column_max_height_raw_risk`
//! - `sum_of_well_depth_raw_penalty` vs `sum_of_well_depth_raw_risk`
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
//! use oxidris_analysis::{
//!     feature_builder::FeatureBuilder, normalization::BoardFeatureNormalizationParamCollection,
//!     sample::RawBoardSample, session::SessionData, statistics::RawFeatureStatistics,
//!     survival::SurvivalStatsMap,
//! };
//! use oxidris_evaluator::board_feature;
//! let sessions: Vec<SessionData> = todo!();
//!
//! // 1. Get feature sources
//! let sources = board_feature::source::all_board_feature_sources();
//!
//! // 2. Extract raw samples from sessions
//! let raw_samples = RawBoardSample::from_sessions(&sources, &sessions);
//!
//! // 3. Compute statistics
//! let raw_stats = RawFeatureStatistics::from_samples(&sources, &raw_samples);
//! let survival_stats = SurvivalStatsMap::collect_all_by_feature_value(&sessions, &sources);
//!
//! // 4. Build normalization parameters
//! let norm_params =
//!     BoardFeatureNormalizationParamCollection::from_stats(&sources, &raw_stats, &survival_stats);
//!
//! // 5. Construct features with runtime normalization
//! let builder = FeatureBuilder::new(norm_params);
//! let features = builder.build_all_features().unwrap();
//! ```

use oxidris_evaluator::board_feature::{
    BoardFeatureSource, BoxedBoardFeature, FeatureSignal,
    source::{
        CenterColumnMaxHeight, ColumnTransitions, EdgeIWellDepth, MaxHeight, NumClearedLines,
        NumHoles, RowTransitions, SumOfHoleDepth, SumOfWellDepth, SurfaceBumpiness,
        SurfaceRoughness, TotalHeight,
    },
    transform::{
        IWellReward, LineClearBonus, RawTransform, RawTransformParam, TableTransform,
        TableTransformParam,
    },
};

use crate::normalization::{
    BoardFeatureNormalizationParam, BoardFeatureNormalizationParamCollection,
};

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum BuildFeatureError {
    #[display("Missing normalization parameters for feature source '{source_id}'")]
    MissingNormalizationParam { source_id: String },
}

#[derive(Debug)]
struct FeatureVecBuilder<'a> {
    raw: bool,
    table_km: bool,
    builder: &'a FeatureBuilder,
    features: Vec<BoxedBoardFeature>,
}

impl<'a> FeatureVecBuilder<'a> {
    fn new(builder: &'a FeatureBuilder, raw: bool, table_km: bool) -> Self {
        Self {
            raw,
            table_km,
            builder,
            features: vec![],
        }
    }

    fn add_raw_penalty<S>(&mut self, source: &S) -> Result<(), BuildFeatureError>
    where
        S: BoardFeatureSource,
    {
        if self.raw {
            self.features
                .push(self.builder.build_raw_penalty_for(source)?);
        }
        Ok(())
    }

    fn add_raw_risk<S>(&mut self, source: &S) -> Result<(), BuildFeatureError>
    where
        S: BoardFeatureSource,
    {
        if self.raw {
            self.features.push(self.builder.build_raw_risk_for(source)?);
        }
        Ok(())
    }

    fn add_table_km<S>(&mut self, source: &S) -> Result<(), BuildFeatureError>
    where
        S: BoardFeatureSource,
    {
        if self.table_km {
            self.features.push(self.builder.build_table_km_for(source)?);
        }
        Ok(())
    }
}

/// Feature builder that constructs features from runtime parameters
#[derive(Debug)]
pub struct FeatureBuilder {
    params: BoardFeatureNormalizationParamCollection,
}

impl FeatureBuilder {
    #[must_use]
    pub fn new(params: BoardFeatureNormalizationParamCollection) -> Self {
        Self { params }
    }

    /// Build all raw board features with runtime normalization
    ///
    /// Constructs only raw-transform features (no table-based transforms).
    /// Includes survival, structure, and score features.
    ///
    /// # Returns
    ///
    /// Vector of boxed board features with raw transformations
    ///
    /// # Errors
    ///
    /// Returns error if normalization parameters are missing for any feature
    pub fn build_raw_features(&self) -> Result<Vec<BoxedBoardFeature>, BuildFeatureError> {
        let mut features = vec![];
        features.extend_from_slice(&self.build_survival_features(true, false)?);
        features.extend_from_slice(&self.build_structure_raw_features()?);
        features.extend_from_slice(&self.build_score_features());
        Ok(features)
    }

    /// Build all board features including both raw and table transforms
    ///
    /// Constructs the complete feature set:
    /// - Raw transforms for all features (survival, structure, score)
    /// - Table transforms for survival features (KM-based)
    ///
    /// # Returns
    ///
    /// Vector of boxed board features with both raw and table transformations
    ///
    /// # Errors
    ///
    /// Returns error if normalization parameters are missing for any feature
    pub fn build_all_features(&self) -> Result<Vec<BoxedBoardFeature>, BuildFeatureError> {
        let mut features = vec![];
        features.extend_from_slice(&self.build_survival_features(true, true)?);
        features.extend_from_slice(&self.build_structure_raw_features()?);
        features.extend_from_slice(&self.build_score_features());
        Ok(features)
    }

    /// Build survival features with optional raw and table transforms
    ///
    /// Constructs survival-critical features:
    /// - Holes (`num_holes`, `sum_of_hole_depth`)
    /// - Height (`max_height`, `center_column_max_height`, `total_height`)
    ///
    /// Features from the same source are placed consecutively.
    ///
    /// # Arguments
    ///
    /// * `raw` - Include raw-transform features (penalty/risk)
    /// * `table_km` - Include table-transform features (KM-based)
    fn build_survival_features(
        &self,
        raw: bool,
        table_km: bool,
    ) -> Result<Vec<BoxedBoardFeature>, BuildFeatureError> {
        let mut builder = FeatureVecBuilder::new(self, raw, table_km);

        builder.add_raw_penalty(&NumHoles)?;
        builder.add_table_km(&NumHoles)?;

        builder.add_raw_penalty(&SumOfHoleDepth)?;
        builder.add_table_km(&SumOfHoleDepth)?;

        builder.add_raw_penalty(&MaxHeight)?;
        builder.add_raw_risk(&MaxHeight)?;
        builder.add_table_km(&MaxHeight)?;

        builder.add_raw_penalty(&CenterColumnMaxHeight)?;
        builder.add_raw_risk(&CenterColumnMaxHeight)?;
        builder.add_table_km(&CenterColumnMaxHeight)?;

        builder.add_raw_penalty(&TotalHeight)?;
        builder.add_table_km(&TotalHeight)?;

        Ok(builder.features)
    }

    /// Build structure features with raw transformations
    ///
    /// Constructs features that measure board structure quality:
    /// - Surface metrics (bumpiness, roughness)
    /// - Transition metrics (row, column)
    /// - Well depth
    ///
    /// # Returns
    ///
    /// Vector of structure features with raw (penalty/risk) transforms
    fn build_structure_raw_features(&self) -> Result<Vec<BoxedBoardFeature>, BuildFeatureError> {
        Ok(vec![
            self.build_raw_penalty_for(&SurfaceBumpiness)?,
            self.build_raw_penalty_for(&SurfaceRoughness)?,
            self.build_raw_penalty_for(&RowTransitions)?,
            self.build_raw_penalty_for(&ColumnTransitions)?,
            self.build_raw_penalty_for(&SumOfWellDepth)?,
            self.build_raw_risk_for(&SumOfWellDepth)?,
        ])
    }

    /// Build score features
    ///
    /// Constructs features that directly relate to game score:
    /// - Line clear bonus (discrete rewards)
    /// - I-piece well reward (strategic bonus)
    ///
    /// # Returns
    ///
    /// Vector of score features (no normalization needed)
    fn build_score_features(&self) -> Vec<BoxedBoardFeature> {
        vec![self.build_line_clear_bonus(), self.build_i_well_reward()]
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
    fn build_raw_penalty_for<S>(&self, source: &S) -> Result<BoxedBoardFeature, BuildFeatureError>
    where
        S: BoardFeatureSource,
    {
        let norm_param = self.get_param(source)?;
        let param = RawTransformParam::new(
            FeatureSignal::Negative,
            norm_param.value_percentiles.p05,
            norm_param.value_percentiles.p95,
        );
        Ok(Box::new(RawTransform::new(
            format!("{}_raw_penalty", source.id()),
            format!("{} (Raw Penalty)", source.name()),
            source.clone_boxed(),
            param,
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
    fn build_raw_risk_for<S>(&self, source: &S) -> Result<BoxedBoardFeature, BuildFeatureError>
    where
        S: BoardFeatureSource,
    {
        let norm_param = self.get_param(source)?;
        let param = RawTransformParam::new(
            FeatureSignal::Negative,
            norm_param.value_percentiles.p75,
            norm_param.value_percentiles.p95,
        );
        Ok(Box::new(RawTransform::new(
            format!("{}_raw_risk", source.id()),
            format!("{} (Raw Risk)", source.name()),
            source.clone_boxed(),
            param,
        )))
    }

    /// Build a table-based KM survival feature for a given source
    ///
    /// Creates a feature that transforms raw values through a lookup table
    /// of Kaplan-Meier median survival times. Each raw feature value maps
    /// to its corresponding median survival time.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Board feature source type
    ///
    /// # Arguments
    ///
    /// * `source` - Feature source to transform
    ///
    /// # Returns
    ///
    /// Boxed board feature with table transformation
    ///
    /// # Errors
    ///
    /// Returns error if normalization parameters are missing for the source
    ///
    /// # Feature ID Format
    ///
    /// `{source_id}_table_km` (e.g., "`num_holes_table_km`")
    ///
    /// # Feature Name Format
    ///
    /// `{source_name} (Table KM)` (e.g., "Number of Holes (Table KM)")
    fn build_table_km_for<S>(&self, source: &S) -> Result<BoxedBoardFeature, BuildFeatureError>
    where
        S: BoardFeatureSource,
    {
        let norm_param = self.get_param(source)?;
        let param = TableTransformParam::new(
            norm_param.survival_table.feature_min_value,
            norm_param.survival_table.normalize_min,
            norm_param.survival_table.normalize_max,
            norm_param.survival_table.median_survival_turns.clone(),
        );
        Ok(Box::new(TableTransform::new(
            format!("{}_table_km", source.id()),
            format!("{} (Table KM)", source.name()),
            source.clone_boxed(),
            param,
        )))
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
    fn get_param<S>(&self, source: &S) -> Result<&BoardFeatureNormalizationParam, BuildFeatureError>
    where
        S: BoardFeatureSource,
    {
        self.params
            .get(source)
            .ok_or_else(|| BuildFeatureError::MissingNormalizationParam {
                source_id: source.id().to_string(),
            })
    }
}
