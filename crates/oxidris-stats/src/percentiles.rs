/// Precomputed percentile values for a dataset.
///
/// This structure stores percentile-value pairs for efficient lookup
/// of commonly used percentile points.
///
/// # Examples
///
/// ```
/// use oxidris_stats::percentiles::Percentiles;
///
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
/// let percentiles = Percentiles::new(&values, &[25.0, 50.0, 75.0]);
///
/// assert_eq!(percentiles.get(50.0), Some(6.0));
/// assert_eq!(percentiles.get(25.0), Some(3.0));
/// ```
#[derive(Debug, Clone)]
pub struct Percentiles {
    /// Percentile-value pairs, sorted by percentile.
    /// Each tuple contains (percentile, value) where percentile is 0.0-100.0.
    values: Vec<(f32, f32)>,
}

impl Percentiles {
    /// Computes percentiles from sorted values.
    ///
    /// # Arguments
    ///
    /// * `sorted_values` - Values sorted in ascending order
    /// * `percentile_points` - The percentile points to compute (e.g., [25.0, 50.0, 75.0])
    ///
    /// # Returns
    ///
    /// A `Percentiles` instance with precomputed values.
    ///
    /// # Panics
    ///
    /// Panics if `sorted_values` is not sorted in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxidris_stats::percentiles::Percentiles;
    ///
    /// let mut values = vec![5.0, 2.0, 8.0, 1.0, 9.0];
    /// values.sort_by(f32::total_cmp);
    /// let percentiles = Percentiles::from_sorted(&values, &[50.0, 90.0]);
    /// ```
    #[must_use]
    pub fn from_sorted(sorted_values: &[f32], percentile_points: &[f32]) -> Self {
        assert!(
            sorted_values.is_sorted_by(|a, b| a <= b),
            "values must be sorted in ascending order"
        );

        let values = percentile_points
            .iter()
            .map(|&p| (p, compute_percentile(sorted_values, p)))
            .collect();
        Self { values }
    }

    /// Computes percentiles from unsorted values.
    ///
    /// This method will sort the values internally before computing percentiles.
    ///
    /// # Arguments
    ///
    /// * `values` - The data points to compute percentiles from
    /// * `percentile_points` - The percentile points to compute (e.g., [25.0, 50.0, 75.0])
    ///
    /// # Returns
    ///
    /// A `Percentiles` instance with precomputed values.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxidris_stats::percentiles::Percentiles;
    ///
    /// let values = vec![5.0, 2.0, 8.0, 1.0, 9.0];
    /// let percentiles = Percentiles::new(&values, &[25.0, 50.0, 75.0]);
    ///
    /// assert_eq!(percentiles.get(50.0), Some(5.0));
    /// ```
    #[must_use]
    pub fn new(values: &[f32], percentile_points: &[f32]) -> Self {
        let mut sorted = values.to_vec();
        sorted.sort_by(f32::total_cmp);
        Self::from_sorted(&sorted, percentile_points)
    }

    /// Gets the value at a specific percentile.
    ///
    /// Returns `None` if the percentile was not precomputed.
    ///
    /// # Arguments
    ///
    /// * `percentile` - The percentile to retrieve (0.0 to 100.0)
    ///
    /// # Returns
    ///
    /// The value at the specified percentile, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxidris_stats::percentiles::Percentiles;
    ///
    /// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    /// let percentiles = Percentiles::new(&values, &[50.0, 95.0]);
    ///
    /// assert_eq!(percentiles.get(50.0), Some(3.0));
    /// assert_eq!(percentiles.get(95.0), Some(5.0));
    /// assert_eq!(percentiles.get(25.0), None); // Not precomputed
    /// ```
    #[must_use]
    pub fn get(&self, percentile: f32) -> Option<f32> {
        self.values.iter().find_map(|(p, value)| {
            if (*p - percentile).abs() < f32::EPSILON {
                Some(*value)
            } else {
                None
            }
        })
    }

    /// Returns an iterator over all (percentile, value) pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxidris_stats::percentiles::Percentiles;
    ///
    /// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    /// let percentiles = Percentiles::new(&values, &[25.0, 50.0, 75.0]);
    ///
    /// for (p, value) in percentiles.iter() {
    ///     println!("P{}: {}", p, value);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = (f32, f32)> + '_ {
        self.values.iter().copied()
    }

    /// Returns all percentile-value pairs as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxidris_stats::percentiles::Percentiles;
    ///
    /// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    /// let percentiles = Percentiles::new(&values, &[50.0]);
    ///
    /// assert_eq!(percentiles.as_slice(), &[(50.0, 3.0)]);
    /// ```
    #[must_use]
    pub fn as_slice(&self) -> &[(f32, f32)] {
        &self.values
    }
}

/// Computes a single percentile value from sorted data.
///
/// This function uses the nearest-rank method (also called "ordinary" percentile).
/// For a dataset with n values, the k-th percentile is the value at position
/// `floor(n * k / 100)`.
///
/// # Arguments
///
/// * `sorted_values` - Values sorted in ascending order
/// * `percentile` - The percentile to compute (0.0 to 100.0)
///
/// # Returns
///
/// The value at the specified percentile. Returns `f32::NAN` if the input is empty.
///
/// # Examples
///
/// ```
/// use oxidris_stats::percentiles::compute_percentile;
///
/// let mut values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// values.sort_by(f32::total_cmp);
///
/// let median = compute_percentile(&values, 50.0);
/// assert_eq!(median, 3.0);
///
/// let p25 = compute_percentile(&values, 25.0);
/// assert_eq!(p25, 2.0);
/// ```
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
#[must_use]
pub fn compute_percentile(sorted_values: &[f32], percentile: f32) -> f32 {
    if sorted_values.is_empty() {
        return f32::NAN;
    }
    let idx = ((sorted_values.len() as f32 * percentile) / 100.0) as usize;
    let idx = idx.min(sorted_values.len() - 1);
    sorted_values[idx]
}
