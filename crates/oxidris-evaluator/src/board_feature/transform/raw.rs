use std::fmt;

use serde::{Deserialize, Serialize};

use crate::board_feature::{
    BoardFeature, BoardFeatureSource, BoxedBoardFeature, FeatureProcessing, FeatureSignal,
};

use crate::placement_analysis::PlacementAnalysis;

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
///   - Used for critical metrics (e.g., `max_height_raw_risk`, `sum_of_well_depth_raw_risk`)
///
/// # Example
///
/// ```rust,no_run
/// use oxidris_evaluator::board_feature::{
///     FeatureSignal,
///     source::NumHoles,
///     transform::{RawTransform, RawTransformParam},
/// };
/// use std::borrow::Cow;
///
/// // Create a penalty feature for holes with P05-P95 normalization
/// let param = RawTransformParam::new(
///     FeatureSignal::Negative, // Lower holes is better
///     2.0,                     // P05 (min)
///     15.0,                    // P95 (max)
/// );
/// let feature = RawTransform::new(
///     "num_holes_raw".to_owned(),
///     "Number of Holes (Raw Penalty)".to_owned(),
///     NumHoles,
///     param,
/// );
/// ```
#[derive(Debug, Clone)]
pub struct RawTransform<S> {
    id: String,
    name: String,
    source: S,
    param: RawTransformParam,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawTransformParam {
    signal: FeatureSignal,
    normalize_min: f32,
    normalize_max: f32,
}

impl<S> RawTransform<S> {
    #[must_use]
    pub fn new(id: String, name: String, source: S, param: RawTransformParam) -> Self {
        Self {
            id,
            name,
            source,
            param,
        }
    }
}

impl RawTransformParam {
    #[must_use]
    pub fn new(signal: FeatureSignal, normalize_min: f32, normalize_max: f32) -> Self {
        Self {
            signal,
            normalize_min,
            normalize_max,
        }
    }
}

impl<S> BoardFeature for RawTransform<S>
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
        FeatureProcessing::RawTransform(self.param.clone())
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
        super::linear_normalize(
            transformed,
            self.param.signal,
            self.param.normalize_min,
            self.param.normalize_max,
        )
    }
}
