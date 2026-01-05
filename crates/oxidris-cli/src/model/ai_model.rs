use std::{fs::File, io::BufReader, path::Path};

use anyhow::Context;
use chrono::{DateTime, Utc};
use oxidris_evaluator::board_feature::{self, BoxedBoardFeature, FeatureProcessing};
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
    pub fn open<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let file = File::open(path)
            .with_context(|| format!("Failed to open AI model file: {}", path.display()))?;

        let reader = BufReader::new(file);
        let model = serde_json::from_reader(reader)
            .with_context(|| format!("Failed to read AI model file: {}", path.display()))?;

        Ok(model)
    }

    pub(crate) fn to_feature_weights(&self) -> anyhow::Result<(Vec<BoxedBoardFeature>, Vec<f32>)> {
        let all_sources = board_feature::all_board_feature_sources();
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
                    tf.processing
                        .apply(tf.id.clone().into(), tf.name.clone().into(), source),
                    tf.weight,
                ))
            })
            .collect()
    }
}
