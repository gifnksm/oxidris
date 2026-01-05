use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

use anyhow::Context;
use chrono::{DateTime, Utc};
use oxidris_evaluator::board_feature::{self, BoxedBoardFeature};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiModel {
    pub name: String,
    pub trained_at: DateTime<Utc>,
    pub final_fitness: f32,
    pub placement_weights: BTreeMap<String, f32>,
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
        let all_features = board_feature::all_board_features();
        self.placement_weights
            .iter()
            .map(
                |(feature_id, weight)| -> anyhow::Result<(BoxedBoardFeature, f32)> {
                    let feature = all_features
                        .iter()
                        .find(|f| f.id() == feature_id)
                        .ok_or_else(|| {
                            anyhow::anyhow!("Feature ID {feature_id} in model not found")
                        })?;
                    Ok((feature.clone_boxed(), *weight))
                },
            )
            .collect()
    }
}
