//! Runtime feature construction from normalization parameters
//!
//! This module provides [`FeatureBuilder`] for constructing board features
//! with normalization parameters computed from actual gameplay data, replacing
//! the compile-time constants in `oxidris_evaluator::board_feature`.
//!
//! # Overview
//!
//! Instead of using hardcoded percentiles (e.g., `NumHoles::P05`, `NumHoles::P95`),
//! this builder computes normalization parameters from session data and constructs
//! features dynamically at runtime.
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

    /// Build a linear penalty feature with P05-P95 normalization range
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

    /// Build a linear risk feature with P75-P95 normalization range (threshold-based)
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

    fn build_num_holes_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&NumHoles)
    }

    fn build_sum_of_hole_depth_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&SumOfHoleDepth)
    }

    fn build_max_height_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&MaxHeight)
    }

    fn build_max_height_linear_risk(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_risk_for(&MaxHeight)
    }

    fn build_center_column_max_height_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&CenterColumnMaxHeight)
    }

    fn build_center_column_max_height_linear_risk(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_risk_for(&CenterColumnMaxHeight)
    }

    fn build_total_height_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&TotalHeight)
    }

    fn build_row_transitions_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&RowTransitions)
    }

    fn build_column_transitions_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&ColumnTransitions)
    }

    fn build_surface_bumpiness_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&SurfaceBumpiness)
    }

    fn build_surface_roughness_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&SurfaceRoughness)
    }

    fn build_well_depth_linear_penalty(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_penalty_for(&SumOfWellDepth)
    }

    fn build_well_depth_linear_risk(&self) -> anyhow::Result<BoxedBoardFeature> {
        self.build_linear_risk_for(&SumOfWellDepth)
    }

    #[expect(clippy::unused_self)]
    fn build_line_clear_bonus(&self) -> BoxedBoardFeature {
        let source = NumClearedLines;
        Box::new(LineClearBonus::new(
            "line_clear_bonus".into(),
            "Line Clear Bonus".into(),
            source,
        ))
    }

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
