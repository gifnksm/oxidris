//! Survival analysis table display
//!
//! This module provides functions for displaying survival statistics
//! in a consistent tabular format.

use super::stats::SurvivalStats;

/// A row in a survival analysis table
pub(super) struct SurvivalTableRow<'a> {
    /// Label for this row (e.g., feature value, phase name, evaluator name)
    pub label: String,
    /// Survival statistics for this row
    pub stats: &'a SurvivalStats,
}

/// Print table header
///
/// # Arguments
/// * `label_col` - Name of the label column (e.g., "Value", "Phase", "Evaluator")
/// * `include_km` - Whether to include KM columns (Median(KM), KM vs All)
fn print_survival_table_header(label_col: &str, include_km: bool) {
    if include_km {
        println!(
            "  {:<20} {:>8} {:>10} {:>12} {:>12} {:>10} {:>12} {:>12}",
            label_col,
            "Boards",
            "Censored%",
            "Mean(Comp)",
            "Mean(All)",
            "All/Comp",
            "Median(KM)",
            "KM vs All",
        );
    } else {
        println!(
            "  {:<20} {:>8} {:>10} {:>12} {:>12} {:>10}",
            label_col, "Boards", "Censored%", "Mean(Comp)", "Mean(All)", "All/Comp",
        );
    }
}

/// Print table separator line
fn print_survival_table_separator(include_km: bool) {
    let total_width = if include_km {
        // label(20) + count(8) + censored%(10) + mean_comp(12) + mean_all(12) + all_comp(10) + median_km(12) + km_vs_all(12) + spaces(7)
        103
    } else {
        // label(20) + count(8) + censored%(10) + mean_comp(12) + mean_all(12) + all_comp(10) + spaces(5)
        87
    };
    println!("  {}", "-".repeat(total_width));
}

/// Print a single table row
///
/// # Arguments
/// * `row` - The row data to print
/// * `include_km` - Whether to include KM columns
fn print_survival_table_row(row: &SurvivalTableRow, include_km: bool) {
    let stats = row.stats;

    if include_km {
        let median_str = stats
            .median_km
            .map_or("N/A".to_string(), |m| format!("{m:.1}"));

        println!(
            "  {:<20} {:>8} {:>9.1}% {:>12.1} {:>12.1} {:>10} {:>12} {:>12}",
            row.label,
            stats.boards_count,
            stats.censoring_rate(),
            stats.mean_complete,
            stats.mean_all,
            stats.all_comp_ratio_str(),
            median_str,
            stats.km_vs_all_str(),
        );
    } else {
        println!(
            "  {:<20} {:>8} {:>9.1}% {:>12.1} {:>12.1} {:>10}",
            row.label,
            stats.boards_count,
            stats.censoring_rate(),
            stats.mean_complete,
            stats.mean_all,
            stats.all_comp_ratio_str(),
        );
    }
}

/// Print a formatted survival statistics table
///
/// # Arguments
/// * `label_col` - Name of the label column
/// * `rows` - Vec of table rows
/// * `include_km` - Whether to include KM columns
pub(super) fn print_survival_table(label_col: &str, rows: Vec<SurvivalTableRow>, include_km: bool) {
    print_survival_table_header(label_col, include_km);
    print_survival_table_separator(include_km);

    for row in rows {
        print_survival_table_row(&row, include_km);
    }
}

/// Print legend explaining table columns
pub(super) fn print_legend() {
    println!("Legend:");
    println!("  Mean(Comp)  : Mean survival of complete games only (censored data excluded)");
    println!("  Mean(All)   : Naive mean of all data (complete + censored, biased estimate)");
    println!("  All/Comp    : Optimistic bias ratio (⚠ when > 1.5)");
    println!("  Median(KM)  : Kaplan-Meier median survival (unbiased estimate handling censoring)");
    println!("  KM vs All   : Difference between KM median and naive mean (% change)");
}
