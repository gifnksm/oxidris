/// Descriptive statistics summarizing a dataset.
///
/// This structure contains common measures of central tendency, dispersion,
/// and spread for a dataset of `f32` values.
#[derive(Debug, Clone)]
pub struct DescriptiveStats {
    /// The minimum value in the dataset.
    pub min: f32,
    /// The maximum value in the dataset.
    pub max: f32,
    /// The arithmetic mean (average) of the dataset.
    pub mean: f32,
    /// The median value of the dataset.
    pub median: f32,
    /// The variance of the dataset.
    pub variance: f32,
    /// The standard deviation of the dataset.
    pub std_dev: f32,
    /// The normalized standard deviation (`std_dev / range`).
    pub normalized_std_dev: f32,
}

impl DescriptiveStats {
    /// Computes descriptive statistics from unsorted values.
    ///
    /// This method will sort the values internally before computing statistics.
    ///
    /// # Arguments
    ///
    /// * `values` - An iterator over `f32` values. The values will be collected and sorted internally.
    ///
    /// # Returns
    ///
    /// * `Some(DescriptiveStats)` - if the dataset contains at least one value
    /// * `None` - if the dataset is empty
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxidris_stats::descriptive::DescriptiveStats;
    /// let values = [5.0, 2.0, 4.0, 1.0, 3.0];
    /// let stats = DescriptiveStats::new(values).unwrap();
    /// assert_eq!(stats.min, 1.0);
    /// assert_eq!(stats.max, 5.0);
    /// assert_eq!(stats.mean, 3.0);
    /// assert_eq!(stats.median, 3.0);
    /// ```
    #[must_use]
    pub fn new<I>(values: I) -> Option<Self>
    where
        I: IntoIterator<Item = f32>,
    {
        let mut values = values.into_iter().collect::<Vec<_>>();
        values.sort_by(f32::total_cmp);
        Self::from_sorted(&values)
    }

    /// Computes descriptive statistics from pre-sorted values.
    ///
    /// This is an optimized version that skips the sorting step.
    /// Use this when you already have sorted data to avoid unnecessary work.
    ///
    /// # Arguments
    ///
    /// * `sorted_values` - Values sorted in ascending order
    ///
    /// # Returns
    ///
    /// * `Some(DescriptiveStats)` - if the dataset contains at least one value
    /// * `None` - if the dataset is empty
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `sorted_values` is not sorted in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxidris_stats::descriptive::DescriptiveStats;
    /// let mut values = [5.0, 2.0, 4.0, 1.0, 3.0];
    /// values.sort_by(f32::total_cmp);
    /// let stats = DescriptiveStats::from_sorted(&values).unwrap();
    /// assert_eq!(stats.min, 1.0);
    /// assert_eq!(stats.max, 5.0);
    /// ```
    #[expect(clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_sorted(sorted_values: &[f32]) -> Option<Self> {
        assert!(
            sorted_values.is_sorted_by(|a, b| a <= b),
            "values must be sorted in ascending order"
        );

        let min = *sorted_values.first()?;
        let max = *sorted_values.last()?;
        let sum = sorted_values.iter().copied().sum::<f32>();
        let count = sorted_values.len();
        let n = count as f32;
        let mean = sum / n;
        let median = sorted_values[sorted_values.len() / 2];
        let variance = sorted_values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>()
            / n;
        let std_dev = variance.sqrt();
        // Use relative epsilon based on the mean to handle different scales
        // This provides scale-adaptive comparison for detecting near-zero ranges
        let normalized_std_dev = if (max - min).abs() < mean.abs() * f32::EPSILON {
            0.0
        } else {
            std_dev / (max - min)
        };

        Some(Self {
            min,
            max,
            mean,
            median,
            variance,
            std_dev,
            normalized_std_dev,
        })
    }
}
