use crate::{descriptive::DescriptiveStats, histogram::Histogram, percentiles::Percentiles};

/// Comprehensive statistical analysis combining multiple measures.
///
/// This structure provides a complete statistical overview of a dataset by combining:
/// - Basic descriptive statistics (mean, median, variance, standard deviation, etc.)
/// - Percentile values for quantile analysis
/// - Histogram for distribution visualization
///
/// # Examples
///
/// ```
/// use oxidris_stats::comprehensive::ComprehensiveStats;
///
/// let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
/// let stats = ComprehensiveStats::new(
///     values,
///     &[25.0, 50.0, 75.0],  // Percentiles to compute
///     5,                     // Number of histogram bins
///     None,                  // Auto-detect min
///     None,                  // Auto-detect max
///     None,                  // No bin width unit
/// ).unwrap();
///
/// assert_eq!(stats.stats.mean, 5.5);
/// assert_eq!(stats.percentiles.get(50.0), Some(6.0));
/// ```
#[derive(Debug, Clone)]
pub struct ComprehensiveStats {
    /// Basic descriptive statistics for the dataset.
    pub stats: DescriptiveStats,
    /// Precomputed percentile values for quick lookup.
    pub percentiles: Percentiles,
    /// Histogram showing the distribution of values across bins.
    pub histogram: Histogram,
}

impl ComprehensiveStats {
    /// Computes comprehensive statistics from unsorted values.
    ///
    /// This method will sort the values internally before computing all statistics.
    ///
    /// # Arguments
    ///
    /// * `values` - The data points to analyze
    /// * `percentile_points` - The percentile points to compute (e.g., [25.0, 50.0, 75.0])
    /// * `hist_num_bins` - The number of main histogram bins (excluding underflow/overflow)
    /// * `hist_min` - Optional explicit minimum value for histogram bounds
    /// * `hist_max` - Optional explicit maximum value for histogram bounds
    /// * `hist_bin_width_unit` - Optional unit for aligning histogram bin widths
    ///
    /// # Returns
    ///
    /// * `Some(ComprehensiveStats)` - if the dataset contains at least one value
    /// * `None` - if the dataset is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use oxidris_stats::comprehensive::ComprehensiveStats;
    ///
    /// let values = [5.0, 2.0, 8.0, 1.0, 9.0, 3.0];
    /// let stats = ComprehensiveStats::new(
    ///     values,
    ///     &[50.0, 95.0],
    ///     10,
    ///     None,
    ///     None,
    ///     None,
    /// ).unwrap();
    ///
    /// assert!(stats.stats.min <= stats.stats.max);
    /// ```
    #[must_use]
    pub fn new<I>(
        values: I,
        percentile_points: &[f32],
        hist_num_bins: usize,
        hist_min: Option<f32>,
        hist_max: Option<f32>,
        hist_bin_width_unit: Option<f32>,
    ) -> Option<Self>
    where
        I: IntoIterator<Item = f32>,
    {
        let mut sorted = values.into_iter().collect::<Vec<_>>();
        sorted.sort_by(f32::total_cmp);
        Self::from_sorted(
            &sorted,
            percentile_points,
            hist_num_bins,
            hist_min,
            hist_max,
            hist_bin_width_unit,
        )
    }

    /// Computes comprehensive statistics from pre-sorted values.
    ///
    /// This is an optimized version that skips the sorting step.
    /// Use this when you already have sorted data to avoid unnecessary work.
    ///
    /// # Arguments
    ///
    /// * `sorted_values` - Values sorted in ascending order
    /// * `percentile_points` - The percentile points to compute (e.g., [25.0, 50.0, 75.0])
    /// * `hist_num_bins` - The number of main histogram bins (excluding underflow/overflow)
    /// * `hist_min` - Optional explicit minimum value for histogram bounds
    /// * `hist_max` - Optional explicit maximum value for histogram bounds
    /// * `hist_bin_width_unit` - Optional unit for aligning histogram bin widths
    ///
    /// # Returns
    ///
    /// * `Some(ComprehensiveStats)` - if the dataset contains at least one value
    /// * `None` - if the dataset is empty
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `sorted_values` is not sorted in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxidris_stats::comprehensive::ComprehensiveStats;
    ///
    /// let mut values = [5.0, 2.0, 8.0, 1.0, 9.0];
    /// values.sort_by(f32::total_cmp);
    ///
    /// let stats = ComprehensiveStats::from_sorted(
    ///     &values,
    ///     &[25.0, 50.0, 75.0],
    ///     5,
    ///     None,
    ///     None,
    ///     None,
    /// ).unwrap();
    ///
    /// assert_eq!(stats.stats.min, 1.0);
    /// assert_eq!(stats.stats.max, 9.0);
    /// ```
    #[must_use]
    pub fn from_sorted(
        sorted_values: &[f32],
        percentile_points: &[f32],
        hist_num_bins: usize,
        hist_min: Option<f32>,
        hist_max: Option<f32>,
        hist_bin_width_unit: Option<f32>,
    ) -> Option<Self> {
        debug_assert!(
            sorted_values.is_sorted_by(|a, b| a <= b),
            "values must be sorted in ascending order"
        );

        let stats = DescriptiveStats::from_sorted(sorted_values)?;
        let percentiles = Percentiles::from_sorted(sorted_values, percentile_points);
        let histogram = Histogram::from_sorted(
            sorted_values,
            hist_num_bins,
            hist_min,
            hist_max,
            hist_bin_width_unit,
        );

        Some(Self {
            stats,
            percentiles,
            histogram,
        })
    }
}
