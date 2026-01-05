use oxidris_evaluator::board_feature::{BoardFeatureValue, BoxedBoardFeature};
use oxidris_stats::comprehensive::ComprehensiveStats;

use crate::analysis::BoardSample;

/// Statistical summary of a feature across multiple board samples
///
/// Contains comprehensive statistics (mean, percentiles, histogram, etc.)
/// computed separately for raw, transformed, and normalized feature values.
/// This enables analysis of feature distributions at each processing stage.
///
/// # Statistics Included
///
/// For each value type (raw, transformed, normalized):
/// - Mean, standard deviation
/// - Percentiles (P1, P5, P10, P25, P50, P75, P90, P95, P99)
/// - Histogram with configurable bins
///
/// # Use Cases
///
/// - Feature normalization parameter tuning
/// - Distribution analysis for feature engineering
/// - Outlier detection and data quality checks
///
/// # Examples
///
/// ```no_run
/// use oxidris_cli::analysis::BoardFeatureStatistics;
/// # let features = todo!();
/// # let samples = todo!();
/// // Compute statistics for all features
/// let stats = BoardFeatureStatistics::from_samples(&features, &samples);
///
/// // Analyze a specific feature
/// println!("Raw mean: {}", stats[0].raw.mean);
/// println!("Normalized P95: {}", stats[0].normalized.percentiles.get(95.0).unwrap());
/// ```
#[derive(Debug, Clone)]
pub struct BoardFeatureStatistics {
    /// Statistics for raw feature values (direct measurements)
    pub raw: ComprehensiveStats,
    /// Statistics for transformed feature values (after non-linear transforms)
    pub transformed: ComprehensiveStats,
    /// Statistics for normalized feature values (scaled to [0, 1])
    pub normalized: ComprehensiveStats,
}

impl BoardFeatureStatistics {
    /// Compute statistics from a feature value iterator
    ///
    /// Calculates comprehensive statistics (percentiles, histogram, etc.)
    /// for raw, transformed, and normalized values of a single feature.
    ///
    /// # Arguments
    ///
    /// * `values` - Iterator over feature values from multiple samples
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidris_cli::analysis::BoardFeatureStatistics;
    /// # let samples = todo!();
    /// # let feature_idx = 0;
    /// let values = samples.iter().map(|s| s.feature_vector[feature_idx]).collect::<Vec<_>>();
    /// let stats = BoardFeatureStatistics::from_feature_values(&values);
    /// ```
    #[expect(clippy::cast_precision_loss)]
    pub fn from_feature_values(values: &[BoardFeatureValue]) -> Self {
        assert!(
            !values.is_empty(),
            "Cannot compute statistics from empty feature values"
        );

        let raw_values = values.iter().map(|f| f.raw as f32);
        let transformed_values = values.iter().map(|f| f.transformed);
        let normalized_values = values.iter().map(|f| f.normalized);

        let percentile_points = &[1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0];
        let hist_num_bins = 11;

        Self {
            raw: ComprehensiveStats::new(
                raw_values,
                percentile_points,
                hist_num_bins,
                Some(0.0),
                None,
                Some(1.0),
            )
            .unwrap(),
            transformed: ComprehensiveStats::new(
                transformed_values,
                percentile_points,
                hist_num_bins,
                Some(0.0),
                None,
                None,
            )
            .unwrap(),
            normalized: ComprehensiveStats::new(
                normalized_values,
                percentile_points,
                hist_num_bins,
                Some(0.0),
                Some(1.0),
                Some(0.1),
            )
            .unwrap(),
        }
    }

    /// Compute statistics for all features across board samples
    ///
    /// Returns one statistics object per feature, computed from
    /// the feature vectors of all board samples.
    ///
    /// # Arguments
    ///
    /// * `features` - Feature definitions (determines the number of statistics objects)
    /// * `board_samples` - Samples to compute statistics from
    ///
    /// # Returns
    ///
    /// A vector of statistics, one for each feature in `features`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidris_cli::analysis::BoardFeatureStatistics;
    /// # let features = todo!();
    /// # let samples = todo!();
    /// let stats = BoardFeatureStatistics::from_samples(&features, &samples);
    /// for (i, feature_stats) in stats.iter().enumerate() {
    ///     println!("Feature {}: mean = {}", i, feature_stats.raw.mean);
    /// }
    /// ```
    pub fn from_samples(
        features: &[BoxedBoardFeature],
        board_samples: &[BoardSample],
    ) -> Vec<Self> {
        (0..features.len())
            .map(|i| {
                let values = board_samples
                    .iter()
                    .map(|bf| bf.feature_vector[i])
                    .collect::<Vec<_>>();
                BoardFeatureStatistics::from_feature_values(&values)
            })
            .collect()
    }
}
