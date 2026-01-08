//! Normalization parameter computation from board statistics
//!
//! This module provides data structures and functions for computing normalization
//! parameters from raw feature statistics extracted from gameplay sessions.
//!
//! # Overview
//!
//! The normalization pipeline works as follows:
//!
//! 1. Extract raw feature values from sessions using [`BoxedBoardFeatureSource`]
//! 2. Compute statistics (percentiles) using [`RawFeatureStatistics`]
//! 3. Build normalization parameters using [`BoardFeatureNormalizationParamCollection`]
//! 4. Use parameters to construct features with [`FeatureBuilder`](crate::feature_builder::FeatureBuilder)
//!
//! # Example
//!
//! ```no_run
//! use oxidris_analysis::{
//!     normalization::BoardFeatureNormalizationParamCollection, sample::RawBoardSample,
//!     session::SessionData, statistics::RawFeatureStatistics, survival::SurvivalStatsMap,
//! };
//! use oxidris_evaluator::board_feature::{self, BoxedBoardFeatureSource};
//!
//! let sessions: Vec<SessionData> = todo!();
//! let feature: BoxedBoardFeatureSource = todo!();
//!
//! let sources = board_feature::source::all_board_feature_sources();
//! let raw_samples = RawBoardSample::from_sessions(&sources, &sessions);
//! let raw_stats = RawFeatureStatistics::from_samples(&sources, &raw_samples);
//! let survival_stats = SurvivalStatsMap::collect_all_by_feature_value(&sessions, &sources);
//!
//! // Build normalization parameters from statistics
//! let norm_params =
//!     BoardFeatureNormalizationParamCollection::from_stats(&sources, &raw_stats, &survival_stats);
//!
//! // Access percentiles for a specific source
//! if let Some(param) = norm_params.get(&feature) {
//!     println!(
//!         "P05: {}, P95: {}",
//!         param.value_percentiles.p05, param.value_percentiles.p95
//!     );
//! }
//! ```

use std::{
    collections::{BTreeMap, HashMap},
    iter,
};

use oxidris_evaluator::board_feature::{BoardFeatureSource, BoxedBoardFeatureSource};

use crate::{statistics::RawFeatureStatistics, survival::SurvivalStatsMap};

/// Collection of normalization parameters for all feature sources
///
/// Maps feature source IDs to their respective normalization parameters.
/// Used by [`FeatureBuilder`](crate::feature_builder::FeatureBuilder) to construct features
/// with runtime-computed normalization ranges.
#[derive(Debug, Clone)]
pub struct BoardFeatureNormalizationParamCollection {
    /// Mapping from feature source ID to its normalization parameters
    pub feature_params: HashMap<String, BoardFeatureNormalizationParam>,
}

/// Normalization parameters for a single feature source
///
/// Contains percentile values used to determine normalization ranges
/// (e.g., P05-P95 for penalties, P75-P95 for risks).
#[derive(Debug, Clone)]
pub struct BoardFeatureNormalizationParam {
    /// Percentile distribution of raw feature values
    pub value_percentiles: ValuePercentiles,
    /// Survival table for the feature source
    pub survival_table: SurvivalTable,
}

/// Percentile values for a feature source
///
/// Contains commonly used percentiles for normalization:
///
/// - P05, P95: Used for penalty features (full range)
/// - P75, P95: Used for risk features (threshold-based)
/// - P50: Median value (for reference)
#[derive(Debug, Clone)]
pub struct ValuePercentiles {
    pub p01: f32,
    pub p05: f32,
    pub p10: f32,
    pub p25: f32,
    pub p50: f32,
    pub p75: f32,
    pub p90: f32,
    pub p95: f32,
    pub p99: f32,
}

/// Survival table for table-based feature transformations
///
/// Contains a lookup table mapping feature values to their Kaplan-Meier
/// median survival times, along with normalization parameters.
///
/// # Table Coverage
///
/// The table covers feature values from `feature_min_value` to
/// `feature_min_value + median_survival_turns.len() - 1` (inclusive).
///
/// Values are typically derived from P05-P95 percentiles of the feature
/// distribution to focus on the statistically significant range.
#[derive(Debug, Clone)]
pub struct SurvivalTable {
    /// Minimum feature value covered by the table
    pub feature_min_value: u32,
    /// KM median survival time for each feature value
    pub median_survival_turns: Vec<f32>,
    /// Minimum survival time (for normalization)
    pub normalize_min: f32,
    /// Maximum survival time (for normalization)
    pub normalize_max: f32,
}

impl BoardFeatureNormalizationParamCollection {
    /// Construct normalization parameters from feature sources and their statistics
    ///
    /// # Arguments
    ///
    /// * `sources` - Feature sources (determines which parameters to compute)
    /// * `raw_stats` - Pre-computed raw feature statistics (one per source)
    /// * `survival_stats` - Pre-computed survival statistics (one per source)
    ///
    /// # Returns
    ///
    /// A collection mapping source IDs to normalization parameters
    #[must_use]
    pub fn from_stats(
        sources: &[BoxedBoardFeatureSource],
        raw_stats: &[RawFeatureStatistics],
        survival_stats: &[SurvivalStatsMap<u32>],
    ) -> Self {
        let feature_params = iter::zip(sources, iter::zip(raw_stats, survival_stats))
            .map(|(source, (raw_stats, survival_stats))| {
                let source_id = source.id();
                let param = BoardFeatureNormalizationParam::from_stats(raw_stats, survival_stats);
                (source_id.to_string(), param)
            })
            .collect();
        Self { feature_params }
    }

    /// Get normalization parameters for a specific feature source
    ///
    /// # Arguments
    ///
    /// * `source` - Feature source
    ///
    /// # Returns
    ///
    /// The normalization parameters if available, or `None` if not found
    pub fn get<S>(&self, source: &S) -> Option<&BoardFeatureNormalizationParam>
    where
        S: BoardFeatureSource,
    {
        self.feature_params.get(source.id())
    }
}

impl BoardFeatureNormalizationParam {
    /// Construct normalization parameters from raw and survival statistics
    ///
    /// # Arguments
    ///
    /// * `raw_stats` - Raw feature statistics (for percentiles)
    /// * `survival_stats` - Survival statistics grouped by feature value
    ///
    /// # Returns
    ///
    /// Normalization parameters containing both value percentiles and survival table
    #[must_use]
    pub fn from_stats(
        raw_stats: &RawFeatureStatistics,
        survival_stats: &SurvivalStatsMap<u32>,
    ) -> Self {
        Self {
            value_percentiles: ValuePercentiles::from_raw_stats(raw_stats),
            survival_table: SurvivalTable::from_survival_stats(survival_stats),
        }
    }
}

impl ValuePercentiles {
    /// Extract percentile values from raw feature statistics
    #[must_use]
    pub fn from_raw_stats(raw_stats: &RawFeatureStatistics) -> Self {
        let percentiles = &raw_stats.raw.percentiles;
        Self {
            p01: percentiles.get(1.0).unwrap(),
            p05: percentiles.get(5.0).unwrap(),
            p10: percentiles.get(10.0).unwrap(),
            p25: percentiles.get(25.0).unwrap(),
            p50: percentiles.get(50.0).unwrap(),
            p75: percentiles.get(75.0).unwrap(),
            p90: percentiles.get(90.0).unwrap(),
            p95: percentiles.get(95.0).unwrap(),
            p99: percentiles.get(99.0).unwrap(),
        }
    }
}

impl SurvivalTable {
    /// Create a survival table from survival statistics
    ///
    /// Builds a lookup table mapping feature values to their Kaplan-Meier
    /// median survival times. The table covers the P05-P95 range of feature
    /// values, with linear interpolation for values without direct KM estimates.
    ///
    /// # Arguments
    ///
    /// * `survival_stats` - Map of feature values to their survival statistics
    ///
    /// # Returns
    ///
    /// A survival table covering P05-P95 range with KM median survival times
    ///
    /// # Algorithm
    ///
    /// 1. Find P05 and P95 feature values to define table range
    /// 2. For each value in [P05, P95]:
    ///    - Use KM median if available
    ///    - Otherwise, linearly interpolate between nearest values with KM medians
    /// 3. Compute `normalize_min/max` from table values
    ///
    /// # Panics
    ///
    /// Panics if `survival_stats` is empty or lacks sufficient data for percentile calculation
    #[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    #[must_use]
    pub fn from_survival_stats(survival_stats: &SurvivalStatsMap<u32>) -> Self {
        let percentiles = survival_stats.filter_by_percentiles(&[0.05, 0.95]);
        let p05_value = **percentiles.first_key_value().unwrap().0;
        let p95_value = **percentiles.last_key_value().unwrap().0;

        let mut median_km_map = BTreeMap::new();
        for (key, stats) in &survival_stats.map {
            if let Some(median_km) = stats.median_km {
                median_km_map.insert(*key, median_km as f32);
            }
        }

        let median_survival_turns = (p05_value..=p95_value)
            .map(|value| {
                if let Some(median_km) = median_km_map.get(&value) {
                    *median_km
                } else {
                    // linear interpolation
                    // find nearest lower and upper keys with median_km
                    let (lower_key, lower_value) =
                        median_km_map.range(..=value).next_back().unwrap();
                    let (upper_key, upper_value) = median_km_map
                        .range(value..)
                        .next()
                        .map(|(k, v)| (*k, *v))
                        .unwrap();
                    let ratio = (value - lower_key) as f32 / (upper_key - lower_key) as f32;
                    lower_value + ratio * (upper_value - lower_value)
                }
            })
            .collect::<Vec<_>>();

        let normalize_min = median_survival_turns
            .iter()
            .copied()
            .min_by(f32::total_cmp)
            .unwrap();
        let normalize_max = median_survival_turns
            .iter()
            .copied()
            .max_by(f32::total_cmp)
            .unwrap();

        Self {
            feature_min_value: p05_value,
            median_survival_turns,
            normalize_min,
            normalize_max,
        }
    }
}
