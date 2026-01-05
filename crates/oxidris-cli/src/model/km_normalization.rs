use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NormalizationParams {
    pub max_turns: usize,
    pub normalization_method: String,
    pub features: BTreeMap<String, FeatureNormalization>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureNormalization {
    pub transform_mapping: BTreeMap<u32, f64>,
    pub normalization: NormalizationRange,
    pub stats: NormalizationStats,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NormalizationRange {
    pub km_min: f64,
    pub km_max: f64,
}

impl NormalizationRange {
    /// Normalize a KM median value to 0-1 range
    #[expect(unused, reason = "may be used later")] // TODO
    pub fn normalize(&self, km_median: f64) -> f64 {
        if (self.km_max - self.km_min).abs() < f64::EPSILON {
            0.5
        } else {
            ((km_median - self.km_min) / (self.km_max - self.km_min)).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NormalizationStats {
    pub p05_feature_value: u32,
    pub p95_feature_value: u32,
    pub p05_km_median: f64,
    pub p95_km_median: f64,
    pub total_unique_values: usize,
}

impl NormalizationStats {
    /// Calculate the KM median range (difference in survival time)
    #[expect(unused, reason = "may be used later")] // TODO
    pub fn km_range(&self) -> f64 {
        self.p05_km_median - self.p95_km_median
    }
}
