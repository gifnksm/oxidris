use std::{fmt, iter};

use oxidris_engine::{BitBoard, Piece};

use crate::{AiType, WeightSet, metrics::Metrics};

pub trait PlacementEvaluator: fmt::Debug {
    fn evaluate_placement(&self, board: &BitBoard, placement: Piece) -> f32;
}

#[derive(Debug, Clone)]
pub struct MetricsBasedPlacementEvaluator {
    weights: WeightSet,
}

impl MetricsBasedPlacementEvaluator {
    #[must_use]
    pub fn new(weights: WeightSet) -> Self {
        Self { weights }
    }

    #[must_use]
    pub fn aggro() -> Self {
        Self::new(WeightSet::AGGRO)
    }

    #[must_use]
    pub fn defensive() -> Self {
        Self::new(WeightSet::DEFENSIVE)
    }

    #[must_use]
    pub fn from_ai_type(ai: AiType) -> Self {
        match ai {
            AiType::Aggro => Self::aggro(),
            AiType::Defensive => Self::defensive(),
        }
    }
}

impl PlacementEvaluator for MetricsBasedPlacementEvaluator {
    #[inline]
    fn evaluate_placement(&self, board: &BitBoard, placement: Piece) -> f32 {
        let metrics = Metrics::measure(board, placement);
        iter::zip(metrics.to_array(), self.weights.to_array())
            .map(|(m, w)| m * w)
            .sum()
    }
}
