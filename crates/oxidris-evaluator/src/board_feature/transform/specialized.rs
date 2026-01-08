use std::{borrow::Cow, fmt};

use crate::{
    board_feature::{
        BoardFeature, BoardFeatureSource, BoxedBoardFeature, FeatureProcessing, FeatureSignal,
    },
    placement_analysis::PlacementAnalysis,
};

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
        super::linear_normalize(transformed, FeatureSignal::Positive, 0.0, 6.0)
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
        super::linear_normalize(transformed, FeatureSignal::Positive, 0.0, 1.0)
    }
}
