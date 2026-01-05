//! Board evaluation features for Tetris.
//!
//! This module provides the feature system for evaluating Tetris board states. Each feature
//! produces a normalized score in \[0.0, 1.0\], where higher is always better (negative features
//! are inverted via [`FeatureSignal::Negative`]).
//!
//! # Feature Architecture
//!
//! The feature system separates concerns into two layers:
//!
//! ## Feature Sources ([`source`] module)
//!
//! Feature sources extract raw measurements from board states and are categorized by what they measure:
//!
//! **Survival Features** - Directly affect game termination:
//! - [`source::NumHoles`] - Count of covered empty cells
//! - [`source::SumOfHoleDepth`] - Cumulative depth of buried holes
//! - [`source::MaxHeight`] - Tallest column height
//! - [`source::CenterColumnMaxHeight`] - Tallest among center 4 columns
//! - [`source::TotalHeight`] - Sum of all column heights
//!
//! **Structure Features** - Affect placement flexibility:
//! - [`source::SurfaceBumpiness`] - Height variation between adjacent columns (first-order)
//! - [`source::SurfaceRoughness`] - Local surface curvature (second-order)
//! - [`source::RowTransitions`] - Horizontal fragmentation
//! - [`source::ColumnTransitions`] - Vertical fragmentation
//! - [`source::SumOfWellDepth`] - Cumulative well depth
//!
//! **Score Features** - Directly contribute to game score:
//! - [`source::NumClearedLines`] - Lines cleared by placement
//! - [`source::EdgeIWellDepth`] - I-piece well quality at board edges
//!
//! ## Feature Types
//!
//! Feature sources are wrapped by feature types that provide transformation and normalization:
//!
//! - [`LinearNormalized`] - Linear transformation with percentile-based normalization
//! - [`LineClearBonus`] - Discrete bonus mapping for line clears
//! - [`IWellReward`] - Triangular reward for optimal I-well depth
//!
//! # Feature Processing Pipeline
//!
//! Each feature processes board states through three steps:
//!
//! 1. **Extract Raw** - [`BoardFeatureSource::extract_raw()`] extracts the raw measurement
//! 2. **Transform** - [`BoardFeature::transform()`] converts raw value to meaningful representation
//! 3. **Normalize** - [`BoardFeature::normalize()`] scales to \[0.0, 1.0\]
//!
//! See [`BoardFeature::compute_feature_value()`] for the complete pipeline.
//!
//! # Trait Overview
//!
//! - [`BoardFeatureSource`] - Extracts raw values from board states (implemented by source types)
//! - [`BoardFeature`] - Complete feature computation including transform and normalize (implemented by feature types)

use std::{borrow::Cow, fmt};

use serde::{Deserialize, Serialize};

use crate::board_feature::source::{EdgeIWellDepth, NumClearedLines};

use crate::{
    board_feature::source::{
        CenterColumnMaxHeight, ColumnTransitions, MaxHeight, NumHoles, RowTransitions,
        SumOfHoleDepth, SumOfWellDepth, SurfaceBumpiness, SurfaceRoughness, TotalHeight,
    },
    placement_analysis::PlacementAnalysis,
};

pub mod source;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

impl BoardFeatureSource for BoxedBoardFeatureSource {
    fn id(&self) -> &str {
        self.as_ref().id()
    }

    fn name(&self) -> &str {
        self.as_ref().name()
    }

    fn clone_boxed(&self) -> BoxedBoardFeatureSource {
        self.as_ref().clone_boxed()
    }

    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        self.as_ref().extract_raw(analysis)
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
    fn feature_processing(&self) -> FeatureProcessing;
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

impl BoardFeature for BoxedBoardFeature {
    fn id(&self) -> &str {
        self.as_ref().id()
    }

    fn name(&self) -> &str {
        self.as_ref().name()
    }

    fn feature_source(&self) -> &dyn BoardFeatureSource {
        self.as_ref().feature_source()
    }

    fn feature_processing(&self) -> FeatureProcessing {
        self.as_ref().feature_processing()
    }

    fn clone_boxed(&self) -> BoxedBoardFeature {
        self.as_ref().clone_boxed()
    }

    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        self.as_ref().extract_raw(analysis)
    }

    fn transform(&self, raw: u32) -> f32 {
        self.as_ref().transform(raw)
    }

    fn normalize(&self, transformed: f32) -> f32 {
        self.as_ref().normalize(transformed)
    }
}

/// Linear normalized feature with percentile-based normalization.
///
/// This is the standard feature type that applies linear transformation and percentile-based
/// normalization to raw values extracted from board states. Most features use this type.
///
/// # Transform
///
/// Linear transformation: `transformed = raw as f32`
///
/// This simple transformation preserves the raw measurement scale but as a floating-point value.
///
/// # Normalization
///
/// Percentile-based linear normalization with configurable range:
///
/// ```text
/// normalized = (transformed - min) / (max - min)
/// normalized = normalized.clamp(0.0, 1.0)
/// if signal == Negative:
///     normalized = 1.0 - normalized
/// ```
///
/// Where `min` and `max` are typically percentiles (P05-P95 or P75-P95) computed from
/// actual gameplay data.
///
/// **Negative signal**: For features where lower raw values are better (e.g., holes, height),
/// the normalized value is inverted so that higher normalized scores are always better.
///
/// # Normalization Ranges
///
/// Two common ranges are used:
///
/// - **P05-P95**: Smooth penalties/rewards across the full observed range
///   - Provides continuous feedback throughout the game
///   - Used for general board quality metrics (e.g., `num_holes`, `surface_bumpiness`)
///
/// - **P75-P95**: Thresholded penalties/rewards for dangerous states
///   - Ignores safe values (below P75), only penalizes high-risk scenarios
///   - Used for critical metrics (e.g., `max_height_linear_risk`, `sum_of_well_depth_linear_risk`)
///
/// # Example
///
/// ```rust,no_run
/// use oxidris_evaluator::board_feature::{LinearNormalized, FeatureSignal, source::NumHoles};
/// use std::borrow::Cow;
///
/// // Create a penalty feature for holes with P05-P95 normalization
/// let feature = LinearNormalized::new(
///     Cow::Borrowed("holes_penalty"),
///     Cow::Borrowed("Holes Penalty"),
///     FeatureSignal::Negative,  // Lower holes is better
///     2.0,   // P05 (min)
///     15.0,  // P95 (max)
///     NumHoles,
/// );
/// ```
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

    fn feature_processing(&self) -> FeatureProcessing {
        FeatureProcessing::LinearNormalized {
            signal: self.signal,
            normalize_min: self.normalize_min,
            normalize_max: self.normalize_max,
        }
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureProcessing {
    LinearNormalized {
        signal: FeatureSignal,
        normalize_min: f32,
        normalize_max: f32,
    },
    LineClearBonus,
    IWellReward,
}

impl FeatureProcessing {
    pub fn apply<S>(
        &self,
        id: Cow<'static, str>,
        name: Cow<'static, str>,
        source: S,
    ) -> BoxedBoardFeature
    where
        S: BoardFeatureSource + Clone + 'static,
    {
        match self {
            Self::LinearNormalized {
                signal,
                normalize_min,
                normalize_max,
            } => Box::new(LinearNormalized::new(
                id,
                name,
                *signal,
                *normalize_min,
                *normalize_max,
                source,
            )) as BoxedBoardFeature,
            Self::LineClearBonus => Box::new(LineClearBonus::new(id, name, source)),
            FeatureProcessing::IWellReward => Box::new(IWellReward::new(id, name, source)),
        }
    }
}

/// Discrete bonus for line clears with strong emphasis on 4-line tetrises.
///
/// This feature encourages clearing multiple lines in a single placement, with strong
/// preference for 4-line clears (tetrises). This is a per-placement action reward,
/// not a cumulative board state metric.
///
/// # Transform
///
/// Discrete mapping with exponential weighting:
/// - 0 lines → 0.0 (no reward)
/// - 1 line → 0.0 (singles discouraged to avoid inefficient clearing)
/// - 2 lines → 1.0 (doubles: minor reward)
/// - 3 lines → 2.0 (triples: moderate reward)
/// - 4 lines → 6.0 (tetrises: major reward, 6× doubles)
///
/// The reward structure strongly favors tetrises to align with optimal Tetris strategy,
/// where maximizing 4-line clears yields both higher scores and better board states.
///
/// # Normalization
///
/// - Range: [0.0, 6.0] (transformed range)
/// - Signal: Positive (more lines cleared is better)
/// - Linear scaling within the transformed range
#[derive(Debug, Clone)]
pub struct LineClearBonus<S> {
    id: Cow<'static, str>,
    name: Cow<'static, str>,
    source: S,
}

impl<S> LineClearBonus<S> {
    #[must_use]
    pub const fn new(id: Cow<'static, str>, name: Cow<'static, str>, source: S) -> Self {
        Self { id, name, source }
    }
}

impl<S> BoardFeature for LineClearBonus<S>
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

    fn feature_processing(&self) -> FeatureProcessing {
        FeatureProcessing::LineClearBonus
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

/// Smooth reward for optimal I-piece well setup at board edges.
///
/// This feature encourages building single-column wells at the board edges (leftmost
/// or rightmost columns) at optimal depth for efficient tetris (4-line clear) execution.
/// Rewards wells around depth 4 while discouraging wells that are too shallow or too deep.
///
/// # Transform
///
/// Triangular function peaked at depth 4:
/// ```text
/// peak = 4.0 (optimal I-well depth)
/// width = 2.0 (controls falloff rate)
/// transformed = (1.0 - |raw - peak| / (peak / width)).clamp(0.0, 1.0)
/// ```
///
/// This gives:
/// - Maximum reward (1.0) at depth 4
/// - Linear falloff for shallower wells (not ready for tetris)
/// - Linear falloff for deeper wells (risky, reduced flexibility)
/// - Zero reward at depth 0 and depth 8+
///
/// # Normalization
///
/// - Range: [0.0, 1.0] (transformed range already in bounds)
/// - Signal: Positive (optimal well depth is better)
/// - Linear scaling (identity function since transformed is already normalized)
///
/// # Rationale
///
/// Balances well depth penalties by recognizing when vertical wells are strategic.
/// Only considers edge wells (columns 0 and 9); center wells are ignored as they
/// are less suitable for tetris strategy.
#[derive(Debug, Clone)]
pub struct IWellReward<S> {
    id: Cow<'static, str>,
    name: Cow<'static, str>,
    source: S,
}

impl<S> IWellReward<S> {
    #[must_use]
    pub const fn new(id: Cow<'static, str>, name: Cow<'static, str>, source: S) -> Self {
        Self { id, name, source }
    }
}

impl<S> BoardFeature for IWellReward<S>
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

    fn feature_processing(&self) -> FeatureProcessing {
        FeatureProcessing::IWellReward
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
