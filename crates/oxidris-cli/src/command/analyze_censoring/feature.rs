//! Feature analysis with Kaplan-Meier survival analysis
//!
//! This module provides functions for analyzing individual features,
//! including data collection, KM curve calculation, display, and CSV export.

use std::{fmt::Write as _, fs, path::Path};

use anyhow::Context;
use oxidris_analysis::survival::SurvivalStatsMap;
use oxidris_evaluator::board_feature::BoxedBoardFeatureSource;

use crate::command::analyze_censoring::table::SurvivalTableRow;

use super::table;

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
    let rows = SurvivalTableRow::from_map(
        percentile_values
            .iter()
            .map(|(value, (percentiles, stats))| ((**value, percentiles), *stats)),
        |(value, percentiles)| {
            let percentiles = percentiles
                .iter()
                .map(|p| format!("P{:}", p * 100.0))
                .collect::<Vec<_>>()
                .join("/");
            format!("{percentiles:<10} {value:>10}")
        },
    );

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
    fs::create_dir_all(dir)?;
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

    fs::write(&csv_path, csv_content)
        .with_context(|| format!("Failed to write CSV file: {}", csv_path.display()))?;
    println!("  KM curves saved to: {}", csv_path.display());

    Ok(())
}
