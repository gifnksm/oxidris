use std::{fmt, iter};

use crate::{
    board_analysis::BoardAnalysis,
    board_feature::{BoardFeatureSource, DynBoardFeatureSource, HolesPenalty, TopOutRisk},
};

pub trait PlacementEvaluator: fmt::Debug + Send + Sync {
    fn evaluate_placement(&self, analysis: &BoardAnalysis) -> f32;
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
    fn evaluate_placement(&self, analysis: &BoardAnalysis) -> f32 {
        iter::zip(&self.features, &self.weights)
            .map(|(f, w)| f.compute_feature_value(&analysis).normalized * w)
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct DumpPlacementEvaluator;

impl PlacementEvaluator for DumpPlacementEvaluator {
    #[inline]
    #[expect(clippy::cast_precision_loss)]
    fn evaluate_placement(&self, analysis: &BoardAnalysis) -> f32 {
        let max_height = <TopOutRisk as BoardFeatureSource>::extract_raw(&analysis);
        let covered_holes = <HolesPenalty as BoardFeatureSource>::extract_raw(&analysis);
        -(max_height as f32) - (covered_holes as f32)
    }
}
