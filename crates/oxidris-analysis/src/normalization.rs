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
//!     session::SessionData, statistics::RawFeatureStatistics,
//! };
//! use oxidris_evaluator::board_feature::{self, BoxedBoardFeatureSource};
//!
//! let sessions: Vec<SessionData> = todo!();
//! let feature: BoxedBoardFeatureSource = todo!();
//!
//! let sources = board_feature::all_board_feature_sources();
//! let raw_samples = RawBoardSample::from_sessions(&sources, &sessions);
//! let raw_stats = RawFeatureStatistics::from_samples(&sources, &raw_samples);
//!
//! // Build normalization parameters from statistics
//! let norm_params = BoardFeatureNormalizationParamCollection::from_stats(&sources, &raw_stats);
//!
//! // Access percentiles for a specific source
//! if let Some(param) = norm_params.get(&feature) {
//!     println!(
//!         "P05: {}, P95: {}",
//!         param.value_percentiles.p05, param.value_percentiles.p95
//!     );
//! }
//! ```

use std::{collections::HashMap, iter};

use oxidris_evaluator::board_feature::{BoardFeatureSource, BoxedBoardFeatureSource};

use crate::statistics::RawFeatureStatistics;

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

impl BoardFeatureNormalizationParamCollection {
    /// Construct normalization parameters from feature sources and their statistics
    ///
    /// # Arguments
    ///
    /// * `sources` - Feature sources (determines which parameters to compute)
    /// * `stats` - Pre-computed raw feature statistics (one per source)
    ///
    /// # Returns
    ///
    /// A collection mapping source IDs to normalization parameters
    #[must_use]
    pub fn from_stats(sources: &[BoxedBoardFeatureSource], stats: &[RawFeatureStatistics]) -> Self {
        let feature_params = iter::zip(sources, stats)
            .map(|(source, stats)| {
                let source_id = source.id();
                let param = BoardFeatureNormalizationParam::from_stats(stats);
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
    /// Construct normalization parameters from raw feature statistics
    #[must_use]
    pub fn from_stats(stats: &RawFeatureStatistics) -> Self {
        Self {
            value_percentiles: ValuePercentiles::from_stats(stats),
        }
    }
}

impl ValuePercentiles {
    /// Extract percentile values from raw feature statistics
    #[must_use]
    pub fn from_stats(stats: &RawFeatureStatistics) -> Self {
        Self {
            p01: stats.raw.percentiles.get(1.0).unwrap(),
            p05: stats.raw.percentiles.get(5.0).unwrap(),
            p10: stats.raw.percentiles.get(10.0).unwrap(),
            p25: stats.raw.percentiles.get(25.0).unwrap(),
            p50: stats.raw.percentiles.get(50.0).unwrap(),
            p75: stats.raw.percentiles.get(75.0).unwrap(),
            p90: stats.raw.percentiles.get(90.0).unwrap(),
            p95: stats.raw.percentiles.get(95.0).unwrap(),
            p99: stats.raw.percentiles.get(99.0).unwrap(),
        }
    }
}
