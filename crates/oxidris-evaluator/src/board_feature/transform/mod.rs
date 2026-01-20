use serde::{Deserialize, Serialize};

pub use self::{raw::*, specialized::*, table::*};
use crate::board_feature::{BoardFeatureSource, BoxedBoardFeature, FeatureSignal};

mod raw;
mod specialized;
mod table;

fn linear_normalize(val: f32, signal: FeatureSignal, min: f32, max: f32) -> f32 {
    let span = max - min;
    let norm = ((val - min) / span).clamp(0.0, 1.0);
    match signal {
        FeatureSignal::Positive => norm,
        FeatureSignal::Negative => 1.0 - norm,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureProcessing {
    RawTransform(RawTransformParam),
    TableTransform(TableTransformParam),
    LineClearBonus,
    IWellReward,
}

impl FeatureProcessing {
    pub fn apply<S>(&self, id: String, name: String, source: S) -> BoxedBoardFeature
    where
        S: BoardFeatureSource + Clone + 'static,
    {
        match self {
            Self::RawTransform(param) => {
                Box::new(RawTransform::new(id, name, source, param.clone()))
            }
            Self::TableTransform(param) => {
                Box::new(TableTransform::new(id, name, source, param.clone()))
            }
            Self::LineClearBonus => Box::new(LineClearBonus::new(id, name, source)),
            Self::IWellReward => Box::new(IWellReward::new(id, name, source)),
        }
    }
}
