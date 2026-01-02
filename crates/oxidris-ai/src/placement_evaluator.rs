use std::{fmt, iter};

use crate::{board_feature::DynBoardFeatureSource, placement_analysis::PlacementAnalysis};

pub trait PlacementEvaluator: fmt::Debug + Send + Sync {
    fn evaluate_placement(&self, analysis: &PlacementAnalysis) -> f32;
}

#[derive(Debug, Clone)]
pub struct FeatureBasedPlacementEvaluator {
    features: Vec<&'static dyn DynBoardFeatureSource>,
    weights: Vec<f32>,
}

impl FeatureBasedPlacementEvaluator {
    #[must_use]
    pub fn new(features: Vec<&'static dyn DynBoardFeatureSource>, weights: Vec<f32>) -> Self {
        assert_eq!(features.len(), weights.len());
        Self { features, weights }
    }
}

impl PlacementEvaluator for FeatureBasedPlacementEvaluator {
    #[inline]
    fn evaluate_placement(&self, analysis: &PlacementAnalysis) -> f32 {
        iter::zip(&self.features, &self.weights)
            .map(|(f, w)| f.compute_feature_value(analysis).normalized * w)
            .sum()
    }
}
