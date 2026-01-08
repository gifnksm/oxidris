use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::board_feature::{BoardFeatureSource, BoxedBoardFeature, FeatureSignal};

pub use self::{raw_transform::*, specialized::*};

mod raw_transform;
mod specialized;

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
    RawTransform {
        signal: FeatureSignal,
        normalize_min: f32,
        normalize_max: f32,
    },
    LineClearBonus,
    IWellReward,
}

impl FeatureProcessing {
    pub fn apply<S>(
        &self,
        id: Cow<'static, str>,
        name: Cow<'static, str>,
        source: S,
    ) -> BoxedBoardFeature
    where
        S: BoardFeatureSource + Clone + 'static,
    {
        match self {
            Self::RawTransform {
                signal,
                normalize_min,
                normalize_max,
            } => Box::new(RawTransform::new(
                id,
                name,
                *signal,
                *normalize_min,
                *normalize_max,
                source,
            )) as BoxedBoardFeature,
            Self::LineClearBonus => Box::new(LineClearBonus::new(id, name, source)),
            FeatureProcessing::IWellReward => Box::new(IWellReward::new(id, name, source)),
        }
    }
}
