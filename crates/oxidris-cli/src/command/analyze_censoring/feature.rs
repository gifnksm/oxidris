//! Feature analysis with Kaplan-Meier survival analysis
//!
//! This module provides functions for analyzing individual features,
//! including data collection, KM curve calculation, display, and CSV export.

use std::{collections::BTreeMap, fmt::Write as _, path::Path};

use anyhow::Context;
use oxidris_evaluator::{
    board_feature::BoxedBoardFeatureSource, placement_analysis::PlacementAnalysis,
};

use super::{
    stats::{self, SurvivalStats},
    table,
};
use crate::model::session::SessionData;

/// Compute survival statistics for all values of a feature
///
/// This function collects data for all feature values and computes full
/// Kaplan-Meier statistics. The results can be reused for display,
/// CSV export, and normalization parameter generation.
///
/// # Arguments
/// * `feature` - The feature to analyze
/// * `sessions` - All gameplay sessions
///
/// # Returns
/// Vector of (`feature_value`, `SurvivalStats`) sorted by feature value
pub(super) fn collect_feature_survival_stats(
    feature: &BoxedBoardFeatureSource,
    sessions: &[SessionData],
    include_km: bool,
) -> BTreeMap<u32, SurvivalStats> {
    stats::collect_survival_stats_by_group(sessions, include_km, |_session, board| {
        let analysis = PlacementAnalysis::from_board(&board.before_placement, board.placement);
        feature.extract_raw(&analysis)
    })
}

/// Display feature statistics with representative percentile values
///
/// Shows a table with P0, P25, P50, P75, P100 percentiles based on
/// cumulative board counts.
///
/// # Arguments
/// * `feature` - The feature to display
/// * `all_stats` - Pre-computed statistics for all feature values
pub(super) fn display_feature_statistics(
    feature: &BoxedBoardFeatureSource,
    all_stats: &BTreeMap<u32, SurvivalStats>,
    include_km: bool,
) {
    let total_values = all_stats.len();

    // Select percentile values to display
    let percentile_values = filter_by_percentiles(all_stats);

    // Display table with representative values
    let rows: Vec<_> = percentile_values
        .iter()
        .map(|(value, stats)| table::SurvivalTableRow {
            label: value.to_string(),
            stats,
        })
        .collect();

    println!("{} ({})", feature.name(), feature.id());
    table::print_survival_table("Value", rows, include_km);
    println!("  (Showing P0, P25, P50, P75, P100 by board count, total values: {total_values})");
}

/// Filter survival statistics to only include percentile values
///
/// Returns a map of feature values to their corresponding survival statistics
/// for P0, P25, P50, P75, and P100 percentiles
#[expect(clippy::cast_precision_loss)]
fn filter_by_percentiles(
    all_stats: &BTreeMap<u32, SurvivalStats>,
) -> BTreeMap<u32, &SurvivalStats> {
    let total_boards = all_stats
        .values()
        .map(|stats| stats.boards_count)
        .sum::<usize>();

    let mut cumulative_boards = 0;
    let mut percentile_values = BTreeMap::new();
    let percentiles = [0.0, 0.25, 0.5, 0.75, 1.0];
    let mut percentile_idx = 0;

    for (value, stats) in all_stats {
        cumulative_boards += stats.boards_count;
        let current_percentile = cumulative_boards as f64 / total_boards as f64;

        while percentile_idx < percentiles.len()
            && current_percentile >= percentiles[percentile_idx]
        {
            percentile_values.insert(*value, stats);
            percentile_idx += 1;
        }
    }

    // Ensure we always have the last value
    if let Some((max_value, max_stats)) = all_stats.last_key_value() {
        percentile_values.insert(*max_value, max_stats);
    }

    percentile_values
}

/// Save KM curves to CSV file for all feature values
///
/// # Arguments
/// * `dir` - Output directory
/// * `feature_id` - Feature identifier for filename
/// * `all_stats` - Statistics with KM curves for all feature values
pub(super) fn save_feature_km_curves(
    dir: &Path,
    feature_id: &str,
    all_stats: &BTreeMap<u32, SurvivalStats>,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let csv_path = dir.join(format!("{feature_id}_km.csv"));
    let mut csv_content = String::from("value,time,survival_prob,at_risk,events\n");

    for (value, stat) in all_stats {
        if let Some(km) = &stat.km_curve {
            for i in 0..km.times.len() {
                writeln!(
                    &mut csv_content,
                    "{},{},{},{},{}",
                    value, km.times[i], km.survival_prob[i], km.at_risk[i], km.events[i]
                )
                .with_context(|| format!("Failed to write CSV data for value {value}"))?;
            }
        }
    }

    std::fs::write(&csv_path, csv_content)
        .with_context(|| format!("Failed to write CSV file: {}", csv_path.display()))?;
    println!("  KM curves saved to: {}", csv_path.display());

    Ok(())
}
