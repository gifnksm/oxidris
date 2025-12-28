use std::{fmt, iter};

use oxidris_engine::{BitBoard, Piece};

use crate::{
    ALL_METRICS, ALL_METRICS_COUNT, AiType, BoardAnalysis, CoveredHolesMetric, MaxHeightMetric,
    MetricSet, MetricSource as _, WeightSet,
};

pub trait PlacementEvaluator: fmt::Debug {
    fn evaluate_placement(&self, board: &BitBoard, placement: Piece) -> f32;
}

#[derive(Debug, Clone)]
pub struct MetricsBasedPlacementEvaluator<'a, const N: usize> {
    metrics: MetricSet<'a, N>,
    weights: WeightSet<N>,
}

impl<'a, const N: usize> MetricsBasedPlacementEvaluator<'a, N> {
    #[must_use]
    pub fn new(metrics: MetricSet<'a, N>, weights: WeightSet<N>) -> Self {
        Self { metrics, weights }
    }
}

impl MetricsBasedPlacementEvaluator<'static, ALL_METRICS_COUNT> {
    #[must_use]
    pub fn from_weights(weights: WeightSet<ALL_METRICS_COUNT>) -> Self {
        Self::new(ALL_METRICS, weights)
    }

    #[must_use]
    pub fn aggro() -> Self {
        Self::from_weights(WeightSet::AGGRO)
    }

    #[must_use]
    pub fn defensive() -> Self {
        Self::from_weights(WeightSet::DEFENSIVE)
    }

    #[must_use]
    pub fn from_ai_type(ai: AiType) -> Self {
        match ai {
            AiType::Aggro => Self::aggro(),
            AiType::Defensive => Self::defensive(),
        }
    }
}

impl<const N: usize> PlacementEvaluator for MetricsBasedPlacementEvaluator<'_, N> {
    #[inline]
    fn evaluate_placement(&self, board: &BitBoard, placement: Piece) -> f32 {
        let metric_values = self.metrics.measure_normalized(board, placement);
        iter::zip(metric_values, self.weights.to_array())
            .map(|(m, w)| m * w)
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
        let max_height = MaxHeightMetric::measure_raw(&analysis);
        let covered_holes = CoveredHolesMetric::measure_raw(&analysis);
        -(max_height as f32) - (covered_holes as f32)
    }
}
