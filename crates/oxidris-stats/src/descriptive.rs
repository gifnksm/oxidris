/// A collection of statistical measures for a dataset.
pub struct Statistics {
    /// The minimum value in the dataset.
    pub min: f32,
    /// The maximum value in the dataset.
    pub max: f32,
    /// The arithmetic mean (average) of the dataset.
    pub mean: f32,
    /// The variance of the dataset.
    pub variance: f32,
    /// The standard deviation of the dataset.
    pub std_dev: f32,
    /// The normalized standard deviation (`std_dev / range`).
    pub normalized_std_dev: f32,
}

impl Statistics {
    /// Computes all statistical measures for the given values.
    ///
    /// This method calculates min, max, mean, variance, standard deviation,
    /// and normalized standard deviation for the provided dataset.
    ///
    /// # Arguments
    ///
    /// * `values` - An iterator over the values. Must be `Clone` to allow multiple passes.
    ///
    /// # Returns
    ///
    /// * `Some(Statistics)` - if the dataset contains at least one value
    /// * `None` - if the dataset is empty
    #[expect(clippy::cast_precision_loss)]
    #[must_use]
    pub fn compute(values: impl IntoIterator<Item = f32> + Clone) -> Option<Self> {
        let (min, max, sum, count) = values.clone().into_iter().fold(
            (f32::INFINITY, f32::NEG_INFINITY, 0.0, 0usize),
            |(min, max, sum, count), x| (f32::min(min, x), f32::max(max, x), sum + x, count + 1),
        );
        if count == 0 {
            return None;
        }
        let mean = sum / (count as f32);
        let variance = compute_variance(mean, values)?;
        let std_dev = compute_std_dev(variance);
        let normalized_std_dev = compute_normalized_std_dev(min, max, mean, std_dev);
        Some(Self {
            min,
            max,
            mean,
            variance,
            std_dev,
            normalized_std_dev,
        })
    }
}

/// Returns the maximum value from an iterator of `f32` values.
///
/// Returns `None` if the iterator is empty.
/// Uses `total_cmp` to handle NaN values consistently.
#[must_use]
pub fn compute_max<I>(values: I) -> Option<f32>
where
    I: IntoIterator<Item = f32>,
{
    values.into_iter().max_by(f32::total_cmp)
}

/// Returns the minimum value from an iterator of `f32` values.
///
/// Returns `None` if the iterator is empty.
/// Uses `total_cmp` to handle NaN values consistently.
#[must_use]
pub fn compute_min<I>(values: I) -> Option<f32>
where
    I: IntoIterator<Item = f32>,
{
    values.into_iter().min_by(f32::total_cmp)
}

/// Computes the arithmetic mean (average) of an iterator of `f32` values.
///
/// Returns `None` if the iterator is empty.
///
/// # Note
///
/// This function does not filter out NaN or infinite values.
/// If such values are present, the result will be NaN or infinite.
#[expect(clippy::cast_precision_loss)]
#[must_use]
pub fn compute_mean(values: impl IntoIterator<Item = f32>) -> Option<f32> {
    let (sum, count) = values
        .into_iter()
        .fold((0.0, 0usize), |(sum, count), x| (sum + x, count + 1));
    if count == 0 {
        return None;
    }
    Some(sum / (count as f32))
}

/// Computes the variance of an iterator of `f32` values.
///
/// The variance is the mean of the squared deviations from the provided mean.
///
/// # Arguments
///
/// * `mean` - The mean value of the dataset (pre-computed)
/// * `values` - An iterator over the values
///
/// # Returns
///
/// * `None` if the iterator is empty
/// * `Some(variance)` otherwise
///
/// # Note
///
/// The mean is provided as a parameter to avoid recomputing it when it's already known.
#[must_use]
pub fn compute_variance(mean: f32, values: impl IntoIterator<Item = f32>) -> Option<f32> {
    compute_mean(values.into_iter().map(|x| (x - mean).powi(2)))
}

/// Computes the standard deviation from variance.
///
/// The standard deviation is the square root of the variance.
///
/// # Arguments
///
/// * `variance` - The variance of the dataset (pre-computed)
///
/// # Returns
///
/// The standard deviation value.
#[must_use]
pub fn compute_std_dev(variance: f32) -> f32 {
    variance.sqrt()
}

/// Computes the normalized standard deviation for a set of values.
///
/// The normalized standard deviation is the standard deviation divided by the range (`max - min`).
/// This provides a scale-independent measure of variability.
///
/// # Arguments
///
/// * `min` - The minimum value in the dataset
/// * `max` - The maximum value in the dataset
/// * `mean` - The mean value of the dataset
/// * `std_dev` - The standard deviation of the dataset (pre-computed)
///
/// # Returns
///
/// Returns `0.0` if the range is effectively zero (all values are the same),
/// otherwise returns the normalized standard deviation (`std_dev / range`).
///
/// # Note
///
/// All statistical measures are provided as parameters to avoid recomputing them
/// when they're already known. This function does not return `Option` as it assumes
/// valid pre-computed statistics are provided.
#[must_use]
pub fn compute_normalized_std_dev(min: f32, max: f32, mean: f32, std_dev: f32) -> f32 {
    let range = max - min;
    // Use relative epsilon based on the mean to handle different scales
    // This provides scale-adaptive comparison for detecting near-zero ranges
    if range.abs() < mean.abs() * f32::EPSILON {
        return 0.0;
    }
    std_dev / range
}
