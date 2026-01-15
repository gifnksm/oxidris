//! Censoring analysis command
//!
//! Analyzes censoring patterns in gameplay data and performs Kaplan-Meier
//! survival analysis to handle censored observations properly.

mod feature;
//mod stats;
mod table;

use std::{collections::BTreeMap, fmt, fs::File, io::Write, ops::Range, path::PathBuf};

use anyhow::Context;
use clap::Args;
use oxidris_analysis::{session::SessionData, survival::SurvivalStatsMap};
use oxidris_evaluator::board_feature::{self, BoxedBoardFeatureSource};

use crate::{
    command::analyze_censoring::table::SurvivalTableRow,
    schema::km_normalization::{
        FeatureNormalization, NormalizationParams, NormalizationRange, NormalizationStats,
    },
    util,
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
    let all_features = board_feature::source::all_board_feature_sources();
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
    let collection = util::read_boards_file(&arg.boards)?;
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
        let stats = SurvivalStatsMap::collect_by_feature_value(sessions, feature);
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
fn generate_normalization_params(
    feature_stats: &[(BoxedBoardFeatureSource, SurvivalStatsMap<u32>)],
    max_turns: usize,
) -> anyhow::Result<NormalizationParams> {
    let mut feature_normalizations = BTreeMap::<String, FeatureNormalization>::new();

    for (feature, all_stats) in feature_stats {
        println!("  Processing: {} ({})", feature.name(), feature.id());

        if all_stats.map.is_empty() {
            anyhow::bail!("No data for feature {}", feature.id());
        }

        let total_unique_values = all_stats.map.len();
        let percentile_values = all_stats.filter_by_percentiles(&[0.05, 0.95]);

        let (&&p05_value, (_, p05_stats)) = percentile_values.first_key_value().unwrap();
        let (&&p95_value, (_, p95_stats)) = percentile_values.last_key_value().unwrap();
        let p05_km = p05_stats.median_km.unwrap();
        let p95_km = p95_stats.median_km.unwrap();

        let transform_mapping = all_stats
            .map
            .iter()
            .filter_map(|(value, stats)| stats.median_km.map(|km| (*value, km)))
            .collect::<BTreeMap<u32, f64>>();

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

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.to_str(), f)
    }
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

fn analyze_by_capture_phase(sessions: &[SessionData], max_turns: usize) {
    let phase_stats = SurvivalStatsMap::collect_by_group(sessions, |_session, board| {
        Phase::from_turn(board.turn, max_turns)
    });

    let rows = SurvivalTableRow::from_map(&phase_stats.map, |phase| {
        let range = format!("{:4?}", phase.range(max_turns));
        format!("{phase:<10} {range:<10}")
    });

    println!("Censoring by Capture Phase");
    table::print_survival_table(&format!("{:<10} {:<10}", "Phase", "Range"), rows, false);
}

fn analyze_by_evaluator(sessions: &[SessionData]) {
    let evaluator_stats = SurvivalStatsMap::collect_by_group(sessions, |session, _board| {
        session.placement_evaluator.clone()
    });

    let rows = SurvivalTableRow::from_map(&evaluator_stats.map, Clone::clone);

    println!("Censoring by Evaluator");
    table::print_survival_table("Evaluator", rows, true);
}
