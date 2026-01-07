//! Survival analysis table display
//!
//! This module provides functions for displaying survival statistics
//! in a consistent tabular format.

use std::fmt::Write as _;

use crate::analysis::survival::SurvivalStats;

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
    let mut header = format!(
        "{:<21} {:>8} {:>10} {:>12} {:>12} {:>10}",
        label_col, "Boards", "Censored%", "Mean(Comp)", "Mean(All)", "All/Comp",
    );
    if include_km {
        write!(&mut header, " {:>12} {:>12}", "Median(KM)", "KM vs All").unwrap();
    }
    println!("  {header}");
}

/// Print table separator line
fn print_survival_table_separator(include_km: bool) {
    // label(20) + count(8) + censored%(10) + mean_comp(12) + mean_all(12) + all_comp(10) + spaces(5)
    let mut total_width = 87;
    if include_km {
        // median_km(12) + km_vs_all(12) + spaces(2)
        total_width += 16;
    }
    println!("  {}", "-".repeat(total_width));
}

/// Print a single table row
///
/// # Arguments
/// * `row` - The row data to print
/// * `include_km` - Whether to include KM columns
#[expect(clippy::cast_precision_loss)]
fn print_survival_table_row(row: &SurvivalTableRow, include_km: bool) {
    let stats = row.stats;

    let censoring_rate = 100.0 * stats.censored_count as f64 / stats.boards_count as f64;
    let all_comp_ratio_str = if stats.boards_count == stats.censored_count {
        "N/A".to_string()
    } else {
        let all_comp_ratio = if stats.mean_complete == 0.0 {
            0.0
        } else {
            stats.mean_all / stats.mean_complete
        };
        if all_comp_ratio > 1.5 {
            format!("{all_comp_ratio:.2} ⚠")
        } else {
            format!("{all_comp_ratio:.2}")
        }
    };

    let mut row = format!(
        "{:<21} {:>8} {:>9.1}% {:>12.1} {:>12.1} {:>10}",
        row.label,
        stats.boards_count,
        censoring_rate,
        stats.mean_complete,
        stats.mean_all,
        all_comp_ratio_str,
    );

    if include_km {
        let median_str = stats
            .median_km
            .map_or("N/A".to_string(), |m| format!("{m:.1}"));

        let km_vs_all_str = stats
            .median_km
            .map(|km| (km - stats.mean_all) / stats.mean_all * 100.0)
            .map_or("N/A".to_string(), |pct| format!("{pct:+.1}%"));

        writeln!(&mut row, " {median_str:>12} {km_vs_all_str:>12}").unwrap();
    }
    println!("  {row}");
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
