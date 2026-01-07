//! Censoring analysis command
//!
//! Analyzes censoring patterns in gameplay data and performs Kaplan-Meier
//! survival analysis to handle censored observations properly.

mod feature;
mod stats;
mod table;

use std::{collections::BTreeMap, fs::File, io::Write, ops::Range, path::PathBuf};

use anyhow::Context;
use clap::Args;
use oxidris_evaluator::board_feature::{self, BoxedBoardFeatureSource};

use self::stats::SurvivalStats;
use crate::{
    command::analyze_censoring::table::SurvivalTableRow,
    model::{
        km_normalization::{
            FeatureNormalization, NormalizationParams, NormalizationRange, NormalizationStats,
        },
        session::{SessionCollection, SessionData},
    },
};

#[derive(Debug, Clone, Args)]
pub(crate) struct AnalyzeCensoringArg {
    /// Path to the boards JSON file
    pub boards: PathBuf,

    /// Feature source IDs to analyze (comma-separated)
    #[arg(long, value_delimiter = ',', default_values = ["num_holes", "sum_of_hole_depth", "max_height", "center_column_max_height", "total_height"])]
    pub features: Vec<String>,

    /// Output directory for KM curve CSV files
    #[arg(long)]
    pub km_output_dir: Option<PathBuf>,

    /// Generate normalization parameters and save to this path
    /// Uses P05-P95 robust KM-based normalization
    #[arg(long)]
    pub normalization_output: Option<PathBuf>,
}

pub(crate) fn run(arg: &AnalyzeCensoringArg) -> anyhow::Result<()> {
    let all_features = board_feature::all_board_feature_sources();
    let target_features = arg
        .features
        .iter()
        .map(|feature_id| {
            all_features
                .iter()
                .find(|f| f.id() == feature_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Feature {feature_id} not found"))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let collection = SessionCollection::open(&arg.boards)?;
    let max_turns = collection.max_turns;
    let sessions = &collection.sessions;

    println!("Censoring Analysis Report (MAX_TURNS={max_turns})");
    println!("==========================================\n");

    table::print_legend();
    println!();

    analyze_overall_censoring(sessions);
    println!();

    analyze_by_capture_phase(sessions, max_turns);
    println!();

    analyze_by_evaluator(sessions);
    println!();

    // Compute statistics for all features (reused for display, CSV, and normalization)
    let mut feature_stats = Vec::new();
    for feature in &target_features {
        let stats = feature::compute_feature_statistics(feature, sessions);
        feature::display_feature_statistics(feature, &stats);

        // Save CSV files if output directory specified
        if let Some(dir) = &arg.km_output_dir {
            feature::save_feature_km_curves(dir, feature.id(), &stats)?;
        }

        println!();
        feature_stats.push((feature.clone(), stats));
    }

    if let Some(output_path) = &arg.normalization_output {
        println!("========================================");
        println!("Generating Normalization Parameters");
        println!("========================================\n");

        println!("Features: {}", arg.features.join(", "));

        let normalization_params = generate_normalization_params(&feature_stats, max_turns)?;

        save_normalization_params(&normalization_params, output_path)?;

        println!(
            "\nNormalization parameters saved to: {}",
            output_path.display()
        );
    }

    Ok(())
}

/// Generate normalization parameters from pre-computed feature statistics
///
/// Reuses the already-computed KM statistics to avoid redundant calculation.
#[expect(clippy::cast_precision_loss)]
fn generate_normalization_params(
    feature_stats: &[(BoxedBoardFeatureSource, Vec<(u32, SurvivalStats)>)],
    max_turns: usize,
) -> anyhow::Result<NormalizationParams> {
    let mut feature_normalizations = BTreeMap::<String, FeatureNormalization>::new();

    for (feature, all_stats) in feature_stats {
        println!("  Processing: {} ({})", feature.name(), feature.id());

        if all_stats.is_empty() {
            anyhow::bail!("No data for feature {}", feature.id());
        }

        // Extract KM medians and board counts from pre-computed statistics
        let mut value_km_data: Vec<(u32, f64, usize)> = vec![];

        for (value, stats) in all_stats {
            if let Some(median) = stats.median_km {
                value_km_data.push((*value, median, stats.count));
            }
        }

        if value_km_data.is_empty() {
            anyhow::bail!("No valid KM medians for feature {}", feature.id());
        }

        let total_unique_values = value_km_data.len();

        // Sort by feature value to calculate cumulative board counts
        value_km_data.sort_by_key(|(value, _, _)| *value);

        // Find P05 and P95 feature values based on board count
        let total_boards: usize = value_km_data.iter().map(|(_, _, count)| count).sum();
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let p05_count = (total_boards as f64 * 0.05) as usize;
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let p95_count = (total_boards as f64 * 0.95) as usize;

        let mut cumulative = 0;
        let mut p05_value = value_km_data[0].0;
        let mut p05_km = value_km_data[0].1;
        let mut p95_value = value_km_data[0].0;
        let mut p95_km = value_km_data[0].1;

        for (value, km_median, count) in &value_km_data {
            if cumulative <= p05_count {
                p05_value = *value;
                p05_km = *km_median;
            }
            if cumulative <= p95_count {
                p95_value = *value;
                p95_km = *km_median;
            }
            cumulative += count;
        }

        // Generate transform mapping (raw value -> KM median)
        let mut transform_mapping = BTreeMap::new();

        for (value, km_median, _) in &value_km_data {
            transform_mapping.insert(*value, *km_median);
        }

        let normalization = NormalizationRange {
            km_min: p95_km,
            km_max: p05_km,
        };

        let stats = NormalizationStats {
            p05_feature_value: p05_value,
            p95_feature_value: p95_value,
            p05_km_median: p05_km,
            p95_km_median: p95_km,
            total_unique_values,
        };

        let feature_norm = FeatureNormalization {
            transform_mapping,
            normalization,
            stats,
        };

        feature_normalizations.insert(feature.id().to_owned(), feature_norm);
    }

    Ok(NormalizationParams {
        max_turns,
        normalization_method: "robust_km".to_string(),
        features: feature_normalizations,
    })
}

fn save_normalization_params(params: &NormalizationParams, path: &PathBuf) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(params)
        .context("Failed to serialize normalization parameters")?;

    let mut file =
        File::create(path).with_context(|| format!("Failed to create file: {}", path.display()))?;

    file.write_all(json.as_bytes())
        .with_context(|| format!("Failed to write to file: {}", path.display()))?;

    Ok(())
}

#[expect(clippy::cast_precision_loss)]
fn analyze_overall_censoring(sessions: &[SessionData]) {
    let total_sessions = sessions.len();
    let censored_sessions = sessions.iter().filter(|s| !s.is_game_over).count();
    let complete_sessions = total_sessions - censored_sessions;
    let total_boards: usize = sessions.iter().map(|s| s.boards.len()).sum();

    println!("Overall Statistics:");
    println!(
        "  Sessions: {} total, {} complete ({:.1}%), {} censored ({:.1}%)",
        total_sessions,
        complete_sessions,
        100.0 * complete_sessions as f64 / total_sessions as f64,
        censored_sessions,
        100.0 * censored_sessions as f64 / total_sessions as f64
    );
    println!("  Total boards captured: {total_boards}");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Phase {
    Early,
    Mid,
    Late,
    VeryLate,
}

impl Phase {
    fn to_str(self) -> &'static str {
        match self {
            Phase::Early => "Early",
            Phase::Mid => "Mid",
            Phase::Late => "Late",
            Phase::VeryLate => "Very Late",
        }
    }

    fn range(self, max_turns: usize) -> Range<usize> {
        match self {
            Phase::Early => 0..max_turns / 4,
            Phase::Mid => max_turns / 4..2 * max_turns / 4,
            Phase::Late => 2 * max_turns / 4..3 * max_turns / 4,
            Phase::VeryLate => 3 * max_turns / 4..max_turns,
        }
    }

    fn from_turn(turn: usize, max_turns: usize) -> Self {
        if turn < max_turns / 4 {
            Phase::Early
        } else if turn < 2 * max_turns / 4 {
            Phase::Mid
        } else if turn < 3 * max_turns / 4 {
            Phase::Late
        } else {
            Phase::VeryLate
        }
    }
}

/// Collect survival data grouped by capture phase
fn collect_phase_data(
    sessions: &[SessionData],
    max_turns: usize,
) -> BTreeMap<Phase, Vec<(usize, bool)>> {
    let mut phase_data = BTreeMap::<Phase, Vec<(usize, bool)>>::new();

    for session in sessions {
        let is_censored = !session.is_game_over;
        let game_end = session.survived_turns;

        for board in &session.boards {
            let phase = Phase::from_turn(board.turn, max_turns);
            let remaining = game_end - board.turn;
            phase_data
                .entry(phase)
                .or_default()
                .push((remaining, is_censored));
        }
    }

    phase_data
}

fn analyze_by_capture_phase(sessions: &[SessionData], max_turns: usize) {
    let phase_data = collect_phase_data(sessions, max_turns);

    let stats_vec: Vec<_> = phase_data
        .values()
        .map(|data| SurvivalStats::from_data(data))
        .collect();

    let rows: Vec<_> = phase_data
        .iter()
        .zip(stats_vec.iter())
        .map(|((phase, _), stats)| SurvivalTableRow {
            label: format!("{:9} {:4?}", phase.to_str(), phase.range(max_turns)),
            stats,
        })
        .collect();

    table::print_survival_table(
        "Censoring by Capture Phase",
        "Phase",
        20,
        "Boards",
        rows,
        false,
        None,
    );
}

/// Collect survival data grouped by evaluator
fn collect_evaluator_data(sessions: &[SessionData]) -> BTreeMap<String, Vec<(usize, bool)>> {
    let mut evaluator_data: BTreeMap<String, Vec<(usize, bool)>> = BTreeMap::new();

    for session in sessions {
        let is_censored = !session.is_game_over;
        let game_end = session.survived_turns;

        evaluator_data
            .entry(session.placement_evaluator.clone())
            .or_default()
            .push((game_end, is_censored));
    }

    evaluator_data
}

fn analyze_by_evaluator(sessions: &[SessionData]) {
    let evaluator_data = collect_evaluator_data(sessions);

    let stats_vec: Vec<_> = evaluator_data
        .values()
        .map(|data| SurvivalStats::from_data(data))
        .collect();

    let rows: Vec<_> = evaluator_data
        .iter()
        .zip(stats_vec.iter())
        .map(|((evaluator, _), stats)| SurvivalTableRow {
            label: evaluator.clone(),
            stats,
        })
        .collect();

    table::print_survival_table(
        "Censoring by Evaluator",
        "Evaluator",
        20,
        "Sessions",
        rows,
        false,
        None,
    );
}
