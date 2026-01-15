use chrono::{DateTime, Utc};
use oxidris_evaluator::board_feature::{self, BoxedBoardFeature, transform::FeatureProcessing};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiModel {
    pub name: String,
    pub trained_at: DateTime<Utc>,
    pub final_fitness: f32,
    pub board_features: Vec<TrainedBoardFeature>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrainedBoardFeature {
    pub id: String,
    pub name: String,
    pub source_id: String,
    pub processing: FeatureProcessing,
    pub weight: f32,
}

impl AiModel {
    pub(crate) fn to_feature_weights(&self) -> anyhow::Result<(Vec<BoxedBoardFeature>, Vec<f32>)> {
        let all_sources = board_feature::source::all_board_feature_sources();
        self.board_features
            .iter()
            .map(|tf| {
                let source = all_sources
                    .iter()
                    .find(|s| s.id() == tf.source_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("Feature source ID {} in model not found", tf.source_id)
                    })?
                    .clone();
                Ok((
                    tf.processing.apply(tf.id.clone(), tf.name.clone(), source),
                    tf.weight,
                ))
            })
            .collect()
    }
}
