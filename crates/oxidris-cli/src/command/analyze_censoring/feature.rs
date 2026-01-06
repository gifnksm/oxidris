//! Feature analysis with Kaplan-Meier survival analysis
//!
//! This module provides functions for analyzing individual features,
//! including data collection, KM curve calculation, display, and CSV export.

use std::{collections::BTreeMap, fmt::Write as _, path::Path};

use anyhow::Context;
use oxidris_evaluator::{
    board_feature::BoxedBoardFeatureSource, placement_analysis::PlacementAnalysis,
};

use super::{stats::SurvivalStats, table};
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
pub(super) fn compute_feature_statistics(
    feature: &BoxedBoardFeatureSource,
    sessions: &[SessionData],
) -> Vec<(u32, SurvivalStats)> {
    let feature_data = collect_feature_data(feature, sessions);

    let mut stats_with_km = Vec::new();
    for (value, data) in feature_data {
        let stats = SurvivalStats::from_data_with_km(&data);
        stats_with_km.push((value, stats));
    }

    stats_with_km
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
    all_stats: &[(u32, SurvivalStats)],
) {
    let total_values = all_stats.len();

    // Select percentile values to display
    let percentile_indices = select_percentile_indices(all_stats);

    // Display table with representative values
    let rows: Vec<_> = percentile_indices
        .iter()
        .filter_map(|&idx| all_stats.get(idx))
        .map(|(value, stats)| table::SurvivalTableRow {
            label: value.to_string(),
            stats,
        })
        .collect();

    let footer =
        format!("(Showing P0, P25, P50, P75, P100 by board count, total values: {total_values})");

    table::print_survival_table(
        &format!("{} ({})", feature.name(), feature.id()),
        "Value",
        8,
        "Boards",
        rows,
        true,
        Some(&footer),
    );
}

/// Collect feature data from all sessions
///
/// Returns a map of `feature_value` -> [(`remaining_turns`, `is_censored`)]
fn collect_feature_data(
    feature: &BoxedBoardFeatureSource,
    sessions: &[SessionData],
) -> BTreeMap<u32, Vec<(usize, bool)>> {
    let mut feature_data: BTreeMap<u32, Vec<(usize, bool)>> = BTreeMap::new();

    for session in sessions {
        let is_censored = !session.is_game_over;
        let game_end = session.survived_turns;

        for board in &session.boards {
            let analysis = PlacementAnalysis::from_board(&board.board, board.placement);
            let raw_value = feature.extract_raw(&analysis);
            let remaining = game_end - board.turn;
            feature_data
                .entry(raw_value)
                .or_default()
                .push((remaining, is_censored));
        }
    }

    feature_data
}

/// Select percentile indices based on cumulative board counts
///
/// Returns indices for P0, P25, P50, P75, P100 percentiles.
#[expect(clippy::cast_precision_loss)]
fn select_percentile_indices(all_stats: &[(u32, SurvivalStats)]) -> Vec<usize> {
    let total_boards: usize = all_stats.iter().map(|(_, stats)| stats.count).sum();

    let mut cumulative_boards = 0;
    let mut percentile_indices = Vec::new();
    let percentiles = [0.0, 0.25, 0.5, 0.75, 1.0];
    let mut percentile_idx = 0;

    for (idx, (_, stats)) in all_stats.iter().enumerate() {
        cumulative_boards += stats.count;
        let current_percentile = cumulative_boards as f64 / total_boards as f64;

        while percentile_idx < percentiles.len()
            && current_percentile >= percentiles[percentile_idx]
        {
            percentile_indices.push(idx);
            percentile_idx += 1;
        }
    }

    // Ensure we always have the last value
    if (percentile_indices.is_empty() || *percentile_indices.last().unwrap() != all_stats.len() - 1)
        && !percentile_indices.contains(&(all_stats.len() - 1))
    {
        percentile_indices.push(all_stats.len() - 1);
    }

    // Deduplicate indices
    percentile_indices.sort_unstable();
    percentile_indices.dedup();

    percentile_indices
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
    all_stats: &[(u32, SurvivalStats)],
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
