use std::ops::Range;

use crate::percentiles;

/// A histogram representation of a dataset's distribution.
///
/// The histogram divides the data range into bins and counts the frequency of values
/// falling into each bin. This implementation uses percentile-based binning (P5-P95)
/// to handle outliers gracefully while preserving tail information in dedicated bins.
#[derive(Debug, Clone)]
pub struct Histogram {
    /// The bins comprising the histogram. May include underflow and overflow bins
    /// at the start and end to capture values outside the main range.
    pub bins: Vec<HistogramBin>,
}

/// A single bin in a histogram.
///
/// Each bin represents a range of values and the count of data points falling within that range.
#[derive(Debug, Clone)]
pub struct HistogramBin {
    /// The range of values covered by this bin (inclusive start, exclusive end).
    pub range: Range<f32>,
    /// The number of values that fall within this bin's range.
    pub count: u64,
}

impl Histogram {
    /// Creates a histogram from unsorted values.
    ///
    /// This method automatically sorts the input values and creates a histogram with
    /// percentile-based binning. The main histogram range covers P5-P95 to avoid
    /// distortion from outliers, with additional underflow/overflow bins capturing
    /// the tails of the distribution.
    ///
    /// # Arguments
    ///
    /// * `values` - The data points to create the histogram from. Will be sorted internally.
    /// * `num_bins` - The number of main bins to create (excluding underflow/overflow bins).
    /// * `explicit_min` - If provided, overrides the minimum value for histogram bounds.
    /// * `explicit_max` - If provided, overrides the maximum value for histogram bounds.
    /// * `bin_width_unit` - If provided, aligns bin widths to multiples of this unit for cleaner display.
    ///
    /// # Returns
    ///
    /// A `Histogram` with bins populated based on the input values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxidris_stats::histogram::Histogram;
    /// let values = [5.0, 2.0, 8.0, 1.0, 9.0, 3.0, 7.0, 4.0, 6.0, 10.0];
    /// let histogram = Histogram::new(values, 5, None, None, None);
    /// assert!(!histogram.bins.is_empty());
    /// ```
    #[must_use]
    pub fn new<I>(
        values: I,
        num_bins: usize,
        explicit_min: Option<f32>,
        explicit_max: Option<f32>,
        bin_width_unit: Option<f32>,
    ) -> Self
    where
        I: IntoIterator<Item = f32>,
    {
        let mut sorted = values.into_iter().collect::<Vec<_>>();
        sorted.sort_by(f32::total_cmp);
        Self::from_sorted(
            &sorted,
            num_bins,
            explicit_min,
            explicit_max,
            bin_width_unit,
        )
    }

    /// Creates a histogram from pre-sorted values.
    ///
    /// This is an optimized version that skips the sorting step.
    /// Use this when you already have sorted data to avoid unnecessary work.
    ///
    /// # Arguments
    ///
    /// * `sorted_values` - Values sorted in ascending order
    /// * `num_bins` - The number of main bins to create (excluding underflow/overflow bins).
    /// * `explicit_min` - If provided, overrides the minimum value for histogram bounds.
    /// * `explicit_max` - If provided, overrides the maximum value for histogram bounds.
    /// * `bin_width_unit` - If provided, aligns bin widths to multiples of this unit for cleaner display.
    ///
    /// # Returns
    ///
    /// A `Histogram` with bins populated based on the input values.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `sorted_values` is not sorted in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxidris_stats::histogram::Histogram;
    /// let mut values = [5.0, 2.0, 8.0, 1.0, 9.0];
    /// values.sort_by(f32::total_cmp);
    /// let histogram = Histogram::from_sorted(&values, 5, None, None, None);
    /// assert!(!histogram.bins.is_empty());
    /// ```
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    #[must_use]
    pub fn from_sorted(
        sorted_values: &[f32],
        num_bins: usize,
        explicit_min: Option<f32>,
        explicit_max: Option<f32>,
        bin_width_unit: Option<f32>,
    ) -> Self {
        assert!(
            sorted_values.is_sorted_by(|a, b| a <= b),
            "values must be sorted in ascending order"
        );

        // Strategy: Use percentile-based clipping (P5–P95) for main bins to avoid distortion
        // from outliers, while capturing underflow/overflow in dedicated bins.
        // This preserves the central distribution shape while retaining tail information.

        if sorted_values.is_empty() || num_bins == 0 {
            return Self { bins: vec![] };
        }

        // Hard bounds: actual data range
        let hard_min = explicit_min
            .unwrap_or_else(|| sorted_values.iter().copied().fold(f32::INFINITY, f32::min));
        let mut hard_max = explicit_max.unwrap_or_else(|| {
            sorted_values
                .iter()
                .copied()
                .fold(f32::NEG_INFINITY, f32::max)
        });

        // Soft bounds: P5 and P95 percentiles used for main histogram range
        // This constrains the visible range to the central 90% distribution
        let soft_min =
            explicit_min.unwrap_or_else(|| percentiles::compute_percentile(sorted_values, 5.0));
        let mut soft_max =
            explicit_max.unwrap_or_else(|| percentiles::compute_percentile(sorted_values, 95.0));

        // Compute range and bin width based on soft bounds
        let mut range = soft_max - soft_min;
        if range < f32::EPSILON {
            // Edge case: distribution is concentrated at a single value
            // Use bin_width_unit or default to 1.0 to avoid division issues
            range = bin_width_unit.unwrap_or(1.0);
        }

        let mut bin_width = range / ((num_bins - 1) as f32);
        if let Some(alignment_unit) = bin_width_unit {
            // Align bin width to unit boundaries for cleaner display
            // ceil() ensures width is a multiple of unit (adds safety margin)
            bin_width = (bin_width / alignment_unit).ceil() * alignment_unit;
            range = bin_width * ((num_bins - 1) as f32);
        }

        soft_max = soft_min + bin_width * ((num_bins - 1) as f32);
        if hard_max < soft_max {
            hard_max = soft_max;
        }

        // Calculate bin boundaries
        // Center first bin at soft_min by offsetting by 0.5 * bin_width
        let first_bin_start = soft_min - 0.5 * bin_width;
        let last_bin_end = first_bin_start + bin_width * num_bins as f32;

        // Detect if data extends beyond the main histogram range
        let has_underflow = hard_min < first_bin_start;
        let has_overflow = hard_max >= last_bin_end;

        let mut bins = vec![];
        if has_underflow {
            // Dedicated bin for values < first_bin_start (left tail / <P5)
            bins.push(HistogramBin {
                range: hard_min..first_bin_start,
                count: 0,
            });
        }
        for bin_idx in 0..num_bins {
            // Recompute bin boundaries to avoid floating-point accumulation errors
            // Use range / (num_bins - 1) instead of accumulated bin_width
            let bin_start = f32::max(
                first_bin_start + (bin_idx as f32) * range / ((num_bins - 1) as f32),
                hard_min,
            );
            let mut bin_end = f32::min(
                first_bin_start + ((bin_idx + 1) as f32) * range / ((num_bins - 1) as f32),
                hard_max,
            );
            // For the last bin without overflow, use next_up() to include values at hard_max boundary
            if !has_overflow && bin_idx == num_bins - 1 {
                bin_end = bin_end.next_up();
            }
            bins.push(HistogramBin {
                range: bin_start..bin_end,
                count: 0,
            });
        }
        if has_overflow {
            // Dedicated bin for values >= last_bin_end (right tail / ≥P95)
            // Use next_up() to ensure hard_max is included
            bins.push(HistogramBin {
                range: last_bin_end..hard_max.next_up(),
                count: 0,
            });
        }

        // Assign each value to its corresponding bin
        for &val in sorted_values {
            let normalized_position = (val - first_bin_start) / bin_width;
            let idx = if normalized_position < 0.0 {
                // Value in underflow region
                assert!(has_underflow, "Underflow detected but no underflow bin");
                0
            } else if normalized_position >= num_bins as f32 {
                // Value in overflow region; offset by 1 if underflow bin exists
                assert!(has_overflow, "Overflow detected but no overflow bin");
                num_bins + usize::from(has_underflow)
            } else {
                // Value in main histogram; offset by 1 if underflow bin exists
                (normalized_position.floor() as usize) + usize::from(has_underflow)
            };
            let bin = &mut bins[idx];
            // assert!(
            //     bin.range.contains(&val),
            //     "value {} not in bin range {:?}",
            //     val,
            //     bin.range
            // );
            bin.count += 1;
        }

        Self { bins }
    }
}
