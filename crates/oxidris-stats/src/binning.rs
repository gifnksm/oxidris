//! Adaptive binning for data analysis
//!
//! This module provides adaptive binning algorithms that create bins based on
//! data distribution rather than fixed intervals. This is particularly useful
//! for handling skewed distributions where fixed-width bins would result in
//! many empty bins or bins with insufficient samples.
//!
//! # Adaptive Binning Algorithm
//!
//! The adaptive binning algorithm creates bins such that each bin contains
//! approximately a target percentage of total samples. This approach:
//!
//! - **Preserves granularity** where sample density is high (typically low values)
//! - **Groups sparse values** together for stable statistics
//! - **Ensures minimum samples** per bin for reliable analysis
//!
//! # Examples
//!
//! ```
//! use oxidris_stats::binning::create_adaptive_bins;
//!
//! // Skewed distribution: many low values, few high values
//! let mut values = vec![0; 40];      // 40 zeros
//! values.extend(vec![1; 30]);        // 30 ones
//! values.extend(vec![2; 20]);        // 20 twos
//! values.extend(vec![100, 101, 102, 103, 104]); // 5 high values
//!
//! // Create bins with 30% per bin (95 total samples, target ~28.5, min 30 applies)
//! let bin_mapping = create_adaptive_bins(&values, 0.30);
//!
//! // Low values get separate bins due to having >30 samples each
//! assert_eq!(bin_mapping[&0].representative, 0);
//! assert_eq!(bin_mapping[&1].representative, 1);
//!
//! // High values are grouped together (only 5 samples total)
//! assert_eq!(bin_mapping[&100].representative, bin_mapping[&104].representative);
//! ```

use std::collections::BTreeMap;

/// Information about an adaptive bin
///
/// Contains the range, sample count, and representative value for a bin
/// created by adaptive binning.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinInfo<K> {
    /// The minimum value in this bin
    pub start: K,
    /// The maximum value in this bin
    pub end: K,
    /// Total number of samples in this bin
    pub count: usize,
    /// Representative value for this bin (midpoint of unique values)
    pub representative: K,
}

impl<K> BinInfo<K> {
    fn from_values(values: &[K], count: usize) -> Self
    where
        K: Ord + Copy,
    {
        let start = *values.first().unwrap();
        let end = *values.last().unwrap();
        // Representative is the midpoint of unique values (median index)
        let representative = values[values.len() / 2];

        BinInfo {
            start,
            end,
            count,
            representative,
        }
    }
}

/// Create adaptive bins for a set of values
///
/// This function analyzes the distribution of values and creates bins such that
/// each bin contains approximately `target_sample_percentage` of total samples.
/// A minimum of 30 samples per bin is enforced to ensure statistical reliability.
///
/// The representative value for each bin is the midpoint of the unique values
/// within that bin, which facilitates linear interpolation for values not in
/// the mapping.
///
/// # Algorithm
///
/// 1. Count occurrences of each unique value
/// 2. Sort values in ascending order
/// 3. Calculate target samples per bin: `max(total_samples × target_percentage, 30)`
/// 4. Accumulate values into bins until reaching target
/// 5. Compute bin info with midpoint representative
///
/// # Arguments
///
/// * `values` - Slice of values to bin
/// * `target_sample_percentage` - Target percentage of samples per bin (0.0 to 1.0)
///
/// # Returns
///
/// A mapping from each unique value to its bin information
///
/// # Examples
///
/// ## Uniform Distribution
///
/// ```
/// use oxidris_stats::binning::create_adaptive_bins;
///
/// let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// let bins = create_adaptive_bins(&values, 0.30);
///
/// // With 10 samples and 30% target, we want 3 samples/bin
/// // Should create ~3 bins
/// let unique_bins: std::collections::HashSet<_> = bins.values().collect();
/// assert!(unique_bins.len() <= 4);
/// ```
///
/// ## Skewed Distribution
///
/// ```
/// use oxidris_stats::binning::create_adaptive_bins;
///
/// // Many samples at value 0, few at others
/// let mut values = vec![0; 50];  // 50 zeros
/// values.extend(vec![1, 2, 3, 4, 5, 100, 101, 102, 103, 104]);
///
/// let bins = create_adaptive_bins(&values, 0.10);
///
/// // Value 0 should get its own bin due to high density
/// assert_eq!(bins[&0].representative, 0);
///
/// // High values should be grouped together
/// assert_eq!(bins[&100].representative, bins[&104].representative);
/// ```
///
/// ## Small Dataset
///
/// ```
/// use oxidris_stats::binning::create_adaptive_bins;
///
/// // Only 3 samples (less than minimum 30)
/// let values = vec![1, 2, 3];
/// let bins = create_adaptive_bins(&values, 0.30);
///
/// // All values in one bin due to minimum constraint
/// assert_eq!(bins[&1].representative, bins[&2].representative);
/// assert_eq!(bins[&2].representative, bins[&3].representative);
/// ```
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
#[must_use]
pub fn create_adaptive_bins<K>(
    values: &[K],
    target_sample_percentage: f64,
) -> BTreeMap<K, BinInfo<K>>
where
    K: Ord + Copy,
{
    if values.is_empty() {
        return BTreeMap::new();
    }

    // Count occurrences of each unique value
    let mut value_counts: BTreeMap<K, usize> = BTreeMap::new();
    for value in values {
        *value_counts.entry(*value).or_insert(0) += 1;
    }

    // Calculate total samples and target per bin
    let total_samples = values.len();
    let target_samples_per_bin =
        ((total_samples as f64 * target_sample_percentage).max(30.0) as usize).min(total_samples);

    // Build bins by accumulating counts and tracking values
    let mut bin_mapping: BTreeMap<K, BinInfo<K>> = BTreeMap::new();
    let mut current_bin_values: Vec<K> = Vec::new();
    let mut current_bin_count = 0;

    for (value, count) in value_counts {
        current_bin_values.push(value);
        current_bin_count += count;

        // Close the bin if we've reached the target
        if current_bin_count >= target_samples_per_bin {
            let bin_info = BinInfo::from_values(&current_bin_values, current_bin_count);
            for &bin_value in &current_bin_values {
                bin_mapping.insert(bin_value, bin_info.clone());
            }

            // Start a new bin
            current_bin_values.clear();
            current_bin_count = 0;
        }
    }

    // Handle the last bin if it has any values
    if !current_bin_values.is_empty() {
        let bin_info = BinInfo::from_values(&current_bin_values, current_bin_count);
        for &bin_value in &current_bin_values {
            bin_mapping.insert(bin_value, bin_info.clone());
        }
    }

    bin_mapping
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_values() {
        let values: Vec<u32> = vec![];
        let bins = create_adaptive_bins(&values, 0.10);
        assert!(bins.is_empty());
    }

    #[test]
    fn test_bin_info_structure() {
        let values = vec![1, 2, 3, 10, 20, 30];
        let bins = create_adaptive_bins(&values, 0.30);

        // Check that BinInfo contains correct information
        for (value, info) in &bins {
            assert!(info.start <= *value);
            assert!(*value <= info.end);
            assert!(info.count > 0);
            assert!(info.start <= info.representative);
            assert!(info.representative <= info.end);
        }
    }

    #[test]
    fn test_single_value() {
        let values = vec![42];
        let bins = create_adaptive_bins(&values, 0.10);
        assert_eq!(bins.len(), 1);
        let info = &bins[&42];
        assert_eq!(info.start, 42);
        assert_eq!(info.end, 42);
        assert_eq!(info.representative, 42);
        assert_eq!(info.count, 1);
    }

    #[test]
    fn test_uniform_distribution() {
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let bins = create_adaptive_bins(&values, 0.30);

        // With 10 samples and 30% target, minimum 30 applies
        // All should be in one bin
        let unique_bins: std::collections::HashSet<_> = bins.values().collect();
        assert_eq!(unique_bins.len(), 1);
    }

    #[test]
    fn test_skewed_distribution() {
        // Many samples at low values, few at high values
        let mut values = vec![0; 50]; // 50 zeros
        values.extend(vec![1; 20]); // 20 ones
        values.extend(vec![2; 10]); // 10 twos
        values.extend(vec![100, 101, 102, 103, 104]); // 5 high values

        // Total: 85 samples, 10% target = 8.5 samples/bin (min 30 applies)
        let bins = create_adaptive_bins(&values, 0.10);

        // Should have multiple bins (count unique representatives)
        let unique_bins: std::collections::HashSet<_> =
            bins.values().map(|info| info.representative).collect();
        assert!(unique_bins.len() >= 2);

        // Value 0 should get its own bin (50 samples > 30)
        let info_0 = &bins[&0];
        assert_eq!(info_0.start, 0);

        // High values should be grouped together (only 5 samples)
        let info_100 = &bins[&100];
        let info_104 = &bins[&104];
        assert_eq!(info_100.representative, info_104.representative);
    }

    #[test]
    fn test_minimum_samples_constraint() {
        // Create dataset with 50 samples, target 5%
        // Target would be 2.5 samples/bin, but minimum 30 should apply
        let values: Vec<u32> = (0..50).collect();
        let bins = create_adaptive_bins(&values, 0.05);

        // Should have ~2 bins (50 / 30)
        let unique_bins: std::collections::HashSet<_> =
            bins.values().map(|info| info.representative).collect();
        assert!(unique_bins.len() <= 2);
    }

    #[test]
    fn test_all_same_value() {
        let values = vec![42; 100];
        let bins = create_adaptive_bins(&values, 0.10);

        assert_eq!(bins.len(), 1);
        let info = &bins[&42];
        assert_eq!(info.representative, 42);
        assert_eq!(info.count, 100);
    }

    #[test]
    fn test_bin_representatives_are_midpoints() {
        let values = vec![1, 2, 3, 10, 20, 30, 100, 200, 300];
        let bins = create_adaptive_bins(&values, 0.30);

        // Each bin representative should be within the bin range
        for (value, info) in &bins {
            assert!(
                info.start <= info.representative && info.representative <= info.end,
                "Bin rep {} should be in range [{}, {}]",
                info.representative,
                info.start,
                info.end
            );
            assert!(
                info.start <= *value && *value <= info.end,
                "Value {} should be in bin range [{}, {}]",
                value,
                info.start,
                info.end
            );
        }
    }

    #[test]
    fn test_with_large_percentage() {
        // 100% target means all values in one bin
        let values = vec![1, 2, 3, 4, 5];
        let bins = create_adaptive_bins(&values, 1.0);

        let unique_bins: std::collections::HashSet<_> =
            bins.values().map(|info| info.representative).collect();
        assert_eq!(unique_bins.len(), 1);
    }

    #[test]
    fn test_preserves_all_values() {
        let values = vec![5, 1, 3, 2, 4, 1, 2, 3, 5, 4];
        let bins = create_adaptive_bins(&values, 0.20);

        // All unique input values should be in the mapping
        for value in &[1, 2, 3, 4, 5] {
            assert!(bins.contains_key(value), "Missing value {value}");
        }
    }
}
