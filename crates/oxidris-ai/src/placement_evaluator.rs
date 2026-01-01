use std::{fmt, iter};

use oxidris_engine::{BitBoard, Piece};

use crate::{
    board_analysis::BoardAnalysis,
    board_feature::{
        ALL_BOARD_FEATURES, ALL_BOARD_FEATURES_COUNT, BoardFeatureSet, BoardFeatureSource as _,
        HolesPenalty, TopOutRisk,
    },
    weights::WeightSet,
};

pub trait PlacementEvaluator: fmt::Debug + Send + Sync {
    fn evaluate_placement(&self, board: &BitBoard, placement: Piece) -> f32;
}

#[derive(Debug, Clone)]
pub struct FeatureBasedPlacementEvaluator<'a, const FEATURE_COUNT: usize> {
    features: BoardFeatureSet<'a, FEATURE_COUNT>,
    weights: WeightSet<FEATURE_COUNT>,
}

impl<'a, const FEATURE_COUNT: usize> FeatureBasedPlacementEvaluator<'a, FEATURE_COUNT> {
    #[must_use]
    pub fn new(
        features: BoardFeatureSet<'a, FEATURE_COUNT>,
        weights: WeightSet<FEATURE_COUNT>,
    ) -> Self {
        Self { features, weights }
    }
}

impl FeatureBasedPlacementEvaluator<'static, ALL_BOARD_FEATURES_COUNT> {
    #[must_use]
    pub fn from_weights(weights: WeightSet<ALL_BOARD_FEATURES_COUNT>) -> Self {
        Self::new(ALL_BOARD_FEATURES, weights)
    }
}

impl<const N: usize> PlacementEvaluator for FeatureBasedPlacementEvaluator<'_, N> {
    #[inline]
    fn evaluate_placement(&self, board: &BitBoard, placement: Piece) -> f32 {
        let feature_values = self.features.measure_normalized(board, placement);
        iter::zip(feature_values, self.weights.as_array())
            .map(|(f, w)| f * w)
            .sum()
    }
}

#[derive(Debug, Clone)]
pub struct DumpPlacementEvaluator;

impl PlacementEvaluator for DumpPlacementEvaluator {
    #[inline]
    #[expect(clippy::cast_precision_loss)]
    fn evaluate_placement(&self, board: &BitBoard, placement: Piece) -> f32 {
        let mut board = board.clone();
        board.fill_piece(placement);
        let analysis = BoardAnalysis::from_board(&board, placement);
        let max_height = TopOutRisk::extract_raw(&analysis);
        let covered_holes = HolesPenalty::extract_raw(&analysis);
        -(max_height as f32) - (covered_holes as f32)
    }
}
