//! Feature analysis with Kaplan-Meier survival analysis
//!
//! This module provides functions for analyzing individual features,
//! including data collection, KM curve calculation, display, and CSV export.

use std::{fmt::Write as _, path::Path};

use anyhow::Context;
use oxidris_analysis::{session::SessionData, survival::SurvivalStatsMap};
use oxidris_evaluator::{
    board_feature::BoxedBoardFeatureSource, placement_analysis::PlacementAnalysis,
};

use super::table;

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
) -> SurvivalStatsMap<u32> {
    SurvivalStatsMap::collect_by_group(sessions, |_session, board| {
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
    all_stats: &SurvivalStatsMap<u32>,
) {
    let total_values = all_stats.map.len();

    // Select percentile values to display
    let percentiles = [0.0, 0.25, 0.5, 0.75, 1.0];
    let percentile_values = all_stats.filter_by_percentiles(&percentiles);

    // Display table with representative values
    let rows: Vec<_> = percentile_values
        .iter()
        .map(|(value, (percentile, stats))| table::SurvivalTableRow {
            label: format!("{:<10} {value:>10}", format!("P{:}", percentile * 100.0),),
            stats,
        })
        .collect();

    println!("{} ({})", feature.name(), feature.id());
    table::print_survival_table(
        &format!("{:<10} {:>10}", "Percentile", "Raw Value"),
        rows,
        true,
    );
    println!("  (Showing P0, P25, P50, P75, P100 by board count, total values: {total_values})");
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
    all_stats: &SurvivalStatsMap<u32>,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let csv_path = dir.join(format!("{feature_id}_km.csv"));
    let mut csv_content = String::from("value,time,survival_prob,at_risk,events\n");

    for (value, stats) in &all_stats.map {
        let km = &stats.km_curve;
        for i in 0..km.times.len() {
            writeln!(
                &mut csv_content,
                "{},{},{},{},{}",
                value, km.times[i], km.survival_prob[i], km.at_risk[i], km.events[i]
            )
            .with_context(|| format!("Failed to write CSV data for value {value}"))?;
        }
    }

    std::fs::write(&csv_path, csv_content)
        .with_context(|| format!("Failed to write CSV file: {}", csv_path.display()))?;
    println!("  KM curves saved to: {}", csv_path.display());

    Ok(())
}
