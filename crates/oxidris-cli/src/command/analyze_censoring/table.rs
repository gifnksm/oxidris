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
/// * `label_width` - Width of the label column
/// * `count_col` - Name of the count column (e.g., "Boards", "Sessions")
/// * `include_km` - Whether to include KM columns (Median(KM), KM vs All)
fn print_survival_table_header(
    label_col: &str,
    label_width: usize,
    count_col: &str,
    include_km: bool,
) {
    if include_km {
        println!(
            "  {:<width$} {:>8} {:>10} {:>12} {:>12} {:>10} {:>12} {:>12}",
            label_col,
            count_col,
            "Censored%",
            "Mean(Comp)",
            "Mean(All)",
            "All/Comp",
            "Median(KM)",
            "KM vs All",
            width = label_width
        );
    } else {
        println!(
            "  {:<width$} {:>8} {:>10} {:>12} {:>12} {:>10}",
            label_col,
            count_col,
            "Censored%",
            "Mean(Comp)",
            "Mean(All)",
            "All/Comp",
            width = label_width
        );
    }
}

/// Print table separator line
fn print_survival_table_separator(label_width: usize, include_km: bool) {
    let total_width = if include_km {
        // label + count(8) + censored%(10) + mean_comp(12) + mean_all(12) + all_comp(10) + median_km(12) + km_vs_all(12) + spaces(7)
        label_width + 83
    } else {
        // label + count(8) + censored%(10) + mean_comp(12) + mean_all(12) + all_comp(10) + spaces(5)
        label_width + 67
    };
    println!("  {}", "-".repeat(total_width));
}

/// Print a single table row
///
/// # Arguments
/// * `row` - The row data to print
/// * `label_width` - Width of the label column
/// * `include_km` - Whether to include KM columns
fn print_survival_table_row(row: &SurvivalTableRow, label_width: usize, include_km: bool) {
    let stats = row.stats;

    if include_km {
        let median_str = stats
            .median_km
            .map_or("N/A".to_string(), |m| format!("{m:.1}"));

        println!(
            "  {:<width$} {:>8} {:>9.1}% {:>12.1} {:>12.1} {:>10} {:>12} {:>12}",
            row.label,
            stats.count,
            stats.censoring_rate(),
            stats.mean_complete,
            stats.mean_all,
            stats.all_comp_ratio_str(),
            median_str,
            stats.km_vs_all_str(),
            width = label_width
        );
    } else {
        println!(
            "  {:<width$} {:>8} {:>9.1}% {:>12.1} {:>12.1} {:>10}",
            row.label,
            stats.count,
            stats.censoring_rate(),
            stats.mean_complete,
            stats.mean_all,
            stats.all_comp_ratio_str(),
            width = label_width
        );
    }
}

/// Print complete table with header, rows, and footer
///
/// # Arguments
/// * `title` - Table title (e.g., "Number of Holes (`num_holes`)")
/// * `label_col` - Name of the label column
/// * `label_width` - Width of the label column
/// * `count_col` - Name of the count column
/// * `rows` - Vec of table rows
/// * `include_km` - Whether to include KM columns
/// * `footer` - Optional footer text (e.g., percentile info)
pub(super) fn print_survival_table(
    title: &str,
    label_col: &str,
    label_width: usize,
    count_col: &str,
    rows: Vec<SurvivalTableRow>,
    include_km: bool,
    footer: Option<&str>,
) {
    println!("{title}");
    print_survival_table_header(label_col, label_width, count_col, include_km);
    print_survival_table_separator(label_width, include_km);

    for row in rows {
        print_survival_table_row(&row, label_width, include_km);
    }

    if let Some(footer_text) = footer {
        println!("  {footer_text}");
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
