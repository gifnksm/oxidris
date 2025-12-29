use std::array;

use oxidris_ai::{ALL_METRICS, MetricMeasurement};

use crate::{data::BoardAndPlacement, tune_metrics::data::HistogramBin};

use super::data::{BoardMetrics, Histogram, MetricStatistics, ValueStats};

pub fn compute_all_metrics(boards: &[BoardAndPlacement]) -> Vec<BoardMetrics> {
    boards.iter().map(compute_board_metrics).collect()
}

fn compute_board_metrics(board: &BoardAndPlacement) -> BoardMetrics {
    let metrics = ALL_METRICS.measure(&board.board, board.placement);
    BoardMetrics {
        board: board.clone(),
        metrics,
    }
}

pub fn coimpute_statistics(
    boards_metrics: &[BoardMetrics],
) -> [MetricStatistics; ALL_METRICS.len()] {
    array::from_fn(|i| compute_metric_statistics(&boards_metrics.iter().map(|bm| bm.metrics[i])))
}

#[expect(clippy::cast_precision_loss)]
fn compute_metric_statistics<I>(values: &I) -> MetricStatistics
where
    I: ExactSizeIterator<Item = MetricMeasurement> + Clone,
{
    let raw_values = values.clone().map(|m| m.raw as f32);
    let transformed_values = values.clone().map(|m| m.transformed);
    let normalized_values = values.clone().map(|m| m.normalized);

    MetricStatistics {
        raw: compute_value_stats(raw_values, 11, Some(0.0), None, Some(1.0)),
        transformed: compute_value_stats(transformed_values, 11, Some(0.0), None, None),
        normalized: compute_value_stats(normalized_values, 11, Some(0.0), Some(1.0), Some(0.1)),
    }
}

#[expect(clippy::cast_precision_loss)]
fn compute_value_stats<I>(
    values: I,
    hist_num_bins: usize,
    hist_min: Option<f32>,
    hist_max: Option<f32>,
    hist_bin_width_unit: Option<f32>,
) -> ValueStats
where
    I: ExactSizeIterator<Item = f32>,
{
    let mut values = values.collect::<Vec<_>>();
    let n = values.len() as f32;
    values.sort_by(f32::total_cmp);

    let min = *values.first().unwrap();
    let max = *values.last().unwrap();
    let mean = values.iter().sum::<f32>() / n;
    let median = values[values.len() / 2];
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    let std_dev = variance.sqrt();

    let percentiles = [1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0]
        .into_iter()
        .map(|p| (p, percentile(&values, p)))
        .collect();

    let histogram = histogram(
        &values,
        hist_num_bins,
        hist_min,
        hist_max,
        hist_bin_width_unit,
    );

    ValueStats {
        min,
        max,
        mean,
        median,
        std_dev,
        percentiles,
        histogram,
    }
}

#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
fn percentile(sorted_values: &[f32], p: f32) -> f32 {
    let idx = ((sorted_values.len() as f32 * p) / 100.0) as usize;
    let idx = idx.min(sorted_values.len() - 1);
    sorted_values[idx]
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub fn histogram(
    values: &[f32],
    num_bins: usize,
    explicit_min: Option<f32>,
    explicit_max: Option<f32>,
    bin_width_unit: Option<f32>,
) -> Histogram {
    // Strategy: Use percentile-based clipping (P5–P95) for main bins to avoid distortion
    // from outliers, while capturing underflow/overflow in dedicated bins.
    // This preserves the central distribution shape while retaining tail information.

    if values.is_empty() || num_bins == 0 {
        return Histogram { bins: vec![] };
    }

    // Hard bounds: actual data range
    let hard_min =
        explicit_min.unwrap_or_else(|| values.iter().copied().fold(f32::INFINITY, f32::min));
    let mut hard_max =
        explicit_max.unwrap_or_else(|| values.iter().copied().fold(f32::NEG_INFINITY, f32::max));

    // Soft bounds: P5 and P95 percentiles used for main histogram range
    // This constrains the visible range to the central 90% distribution
    let soft_min = explicit_min.unwrap_or_else(|| percentile(values, 5.0));
    let mut soft_max = explicit_max.unwrap_or_else(|| percentile(values, 95.0));

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
    for &val in values {
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

    Histogram { bins }
}
