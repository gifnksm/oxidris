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
        raw: compute_value_stats(raw_values, 10, Some(0.0), None, Some(1.0)),
        transformed: compute_value_stats(transformed_values, 10, Some(0.0), None, None),
        normalized: compute_value_stats(normalized_values, 10, Some(0.0), Some(1.0), Some(0.1)),
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
    min: Option<f32>,
    max: Option<f32>,
    bin_width_unit: Option<f32>,
) -> Histogram {
    if values.is_empty() || num_bins == 0 {
        return Histogram { bins: vec![] };
    }

    let min = min.unwrap_or_else(|| values.iter().copied().fold(f32::INFINITY, f32::min));
    let max = max.unwrap_or_else(|| values.iter().copied().fold(f32::NEG_INFINITY, f32::max));

    let range = if (max - min).abs() < f32::EPSILON {
        1.0
    } else {
        max - min
    };

    let mut bin_width = range / num_bins as f32;
    if let Some(unit) = bin_width_unit {
        bin_width = (bin_width / unit).ceil() * unit;
    }
    let mut bins = vec![0; num_bins];

    for &val in values {
        let val = val.clamp(min, max);
        let mut idx = ((val - min) / bin_width).floor() as usize;
        if idx >= num_bins {
            idx = num_bins - 1;
        }
        bins[idx] += 1;
    }

    let bins = bins
        .into_iter()
        .enumerate()
        .map(|(i, count)| HistogramBin {
            range: (min + i as f32 * bin_width)..(min + (i + 1) as f32 * bin_width),
            count,
        })
        .collect();

    Histogram { bins }
}
