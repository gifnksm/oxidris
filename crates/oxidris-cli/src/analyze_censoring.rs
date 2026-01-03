use std::{
    collections::{BTreeMap, HashMap},
    fmt::Write as _,
    fs::File,
    io::Write,
    path::PathBuf,
};

use anyhow::Context;
use clap::Args;
use oxidris_ai::{board_feature::ALL_BOARD_FEATURES, placement_analysis::PlacementAnalysis};
use oxidris_stats::survival::KaplanMeierCurve;

use crate::data::{
    self, FeatureNormalization, NormalizationParams, NormalizationRange, NormalizationStats,
};

#[derive(Debug, Clone, Args)]
pub(crate) struct AnalyzeCensoringArg {
    /// Path to the boards JSON file
    pub boards: PathBuf,

    /// Feature IDs to analyze (comma-separated)
    #[arg(long, value_delimiter = ',', default_values = ["holes_penalty", "max_height_penalty", "hole_depth_penalty"])]
    pub features: Vec<String>,

    /// Perform Kaplan-Meier survival analysis
    #[arg(long)]
    pub kaplan_meier: bool,

    /// Output directory for KM curve CSV files
    #[arg(long)]
    pub km_output_dir: Option<PathBuf>,

    /// Generate normalization parameters and save to this path
    /// Uses P05-P95 robust KM-based normalization
    #[arg(long)]
    pub normalization_output: Option<PathBuf>,
}

pub(crate) fn run(arg: &AnalyzeCensoringArg) -> anyhow::Result<()> {
    let collection = data::load_session_collection(&arg.boards)?;
    let max_turns = collection.max_turns;
    let sessions = &collection.sessions;

    println!("Censoring Analysis Report (MAX_TURNS={max_turns})");
    println!("==========================================\n");

    analyze_overall_censoring(sessions);
    println!();

    analyze_by_capture_phase(sessions, max_turns);
    println!();

    analyze_by_evaluator(sessions);
    println!();

    for feature_id in &arg.features {
        analyze_by_feature(sessions, feature_id)?;
        println!();
    }

    if arg.kaplan_meier {
        println!("========================================");
        println!("Kaplan-Meier Survival Analysis");
        println!("========================================\n");

        for feature_id in &arg.features {
            analyze_feature_survival(sessions, feature_id, arg.km_output_dir.as_ref())?;
            println!();
        }
    }

    if let Some(output_path) = &arg.normalization_output {
        println!("========================================");
        println!("Generating Normalization Parameters");
        println!("========================================\n");

        println!("Features: {}", arg.features.join(", "));

        let normalization_params =
            generate_normalization_params(sessions, max_turns, &arg.features)?;

        save_normalization_params(&normalization_params, output_path)?;

        println!(
            "\nNormalization parameters saved to: {}",
            output_path.display()
        );
    }

    Ok(())
}

#[expect(clippy::cast_precision_loss)]
fn generate_normalization_params(
    sessions: &[data::SessionData],
    max_turns: usize,
    features: &[String],
) -> anyhow::Result<NormalizationParams> {
    let mut feature_normalizations = BTreeMap::new();

    for feature_id in features {
        let feature = ALL_BOARD_FEATURES
            .iter()
            .find(|f| f.id() == feature_id)
            .ok_or_else(|| anyhow::anyhow!("Feature {feature_id} not found"))?;

        println!("  Processing: {} ({})", feature.name(), feature_id);

        // Collect survival data for each feature value
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

        if feature_data.is_empty() {
            anyhow::bail!("No data for feature {feature_id}");
        }

        // Calculate KM median for each feature value
        let mut value_km_data: Vec<(u32, f64, usize)> = Vec::new();

        for (value, data) in &feature_data {
            let km = KaplanMeierCurve::from_data(data.clone());
            if let Some(median) = km.median_survival() {
                value_km_data.push((*value, median, data.len()));
            }
        }

        if value_km_data.is_empty() {
            anyhow::bail!("No valid KM medians for feature {feature_id}");
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

        feature_normalizations.insert(feature_id.clone(), feature_norm);
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
fn analyze_overall_censoring(sessions: &[crate::data::SessionData]) {
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

#[expect(clippy::cast_precision_loss)]
fn analyze_by_capture_phase(sessions: &[crate::data::SessionData], max_turns: usize) {
    println!("Censoring by Capture Phase:");
    println!(
        "  {:<20} {:>8} {:>10} {:>12} {:>12} {:>8}",
        "Phase", "Boards", "Censored%", "Mean(Comp)", "Mean(All)", "Bias"
    );
    println!("  {}", "-".repeat(78));

    // Define phases dynamically based on max_turns
    let phase_size = max_turns / 4;
    let phases = [
        ("Early", 0, phase_size),
        ("Mid", phase_size, phase_size * 2),
        ("Late", phase_size * 2, phase_size * 3),
        ("Very Late", phase_size * 3, max_turns),
    ];

    for (phase_name, start, end) in phases {
        let mut phase_boards = vec![];

        for session in sessions {
            let is_censored = !session.is_game_over;
            let game_end = session.survived_turns;

            for board in &session.boards {
                if board.turn >= start && board.turn < end {
                    let remaining = game_end - board.turn;
                    phase_boards.push((remaining, is_censored));
                }
            }
        }

        if phase_boards.is_empty() {
            continue;
        }

        let total = phase_boards.len();
        let censored = phase_boards.iter().filter(|(_, c)| *c).count();
        let censoring_rate = 100.0 * censored as f64 / total as f64;

        let complete_remaining: Vec<usize> = phase_boards
            .iter()
            .filter(|(_, c)| !*c)
            .map(|(r, _)| *r)
            .collect();

        let all_remaining: Vec<usize> = phase_boards.iter().map(|(r, _)| *r).collect();

        let mean_complete = if complete_remaining.is_empty() {
            0.0
        } else {
            complete_remaining.iter().sum::<usize>() as f64 / complete_remaining.len() as f64
        };

        let mean_all = all_remaining.iter().sum::<usize>() as f64 / all_remaining.len() as f64;

        let bias = if complete_remaining.is_empty() {
            "-".to_string()
        } else {
            format!("{:.2}x", mean_all / mean_complete)
        };

        println!(
            "  {:<20} {:>8} {:>9.1}% {:>12.1} {:>12.1} {:>8}",
            format!("{phase_name} ({start}-{end})"),
            total,
            censoring_rate,
            mean_complete,
            mean_all,
            bias
        );
    }
}

#[expect(clippy::cast_precision_loss)]
fn analyze_feature_survival(
    sessions: &[crate::data::SessionData],
    feature_id: &str,
    output_dir: Option<&PathBuf>,
) -> anyhow::Result<()> {
    // Find the feature
    let feature = ALL_BOARD_FEATURES
        .iter()
        .find(|f| f.id() == feature_id)
        .ok_or_else(|| anyhow::anyhow!("Feature {feature_id} not found"))?;

    println!("Kaplan-Meier Analysis: {} ({})", feature.name(), feature_id);

    // Collect data: feature_value -> [(remaining_turns, is_censored)]
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

    // Show summary table
    println!(
        "  {:<8} {:>8} {:>10} {:>12} {:>12} {:>12}",
        "Value", "Boards", "Censored%", "Median(KM)", "Mean(Naive)", "Diff"
    );
    println!("  {}", "-".repeat(74));

    let total_values = feature_data.len();

    // Calculate cumulative board counts for percentile-based selection
    let all_values: Vec<_> = feature_data.iter().collect();
    let total_boards: usize = all_values.iter().map(|(_, data)| data.len()).sum();

    let mut cumulative_boards = 0;
    let mut percentile_indices = Vec::new();
    let percentiles = [0.0, 0.25, 0.5, 0.75, 1.0];
    let mut percentile_idx = 0;

    for (idx, (_, data)) in all_values.iter().enumerate() {
        cumulative_boards += data.len();
        let current_percentile = cumulative_boards as f64 / total_boards as f64;

        while percentile_idx < percentiles.len()
            && current_percentile >= percentiles[percentile_idx]
        {
            percentile_indices.push(idx);
            percentile_idx += 1;
        }
    }

    // Ensure we always have the last value
    if (percentile_indices.is_empty()
        || *percentile_indices.last().unwrap() != all_values.len() - 1)
        && !percentile_indices.contains(&(all_values.len() - 1))
    {
        percentile_indices.push(all_values.len() - 1);
    }

    // Deduplicate indices
    percentile_indices.sort_unstable();
    percentile_indices.dedup();

    let mut km_curves = Vec::new();

    for &idx in &percentile_indices {
        if let Some((value, data)) = all_values.get(idx) {
            let total = data.len();
            let censored = data.iter().filter(|(_, c)| *c).count();
            let censoring_rate = 100.0 * censored as f64 / total as f64;

            // Calculate KM curve
            let km = KaplanMeierCurve::from_data((*data).clone());
            let median_km = km.median_survival();

            // Naive mean (all data)
            let mean_naive = data.iter().map(|(t, _)| *t).sum::<usize>() as f64 / total as f64;

            let median_str = median_km.map_or("N/A".to_string(), |m| format!("{m:.1}"));
            let diff_str = median_km.map_or("N/A".to_string(), |m| {
                let diff_pct = ((m - mean_naive) / mean_naive * 100.0).abs();
                format!("{diff_pct:.1}%")
            });

            println!(
                "  {value:<8} {total:>8} {censoring_rate:>9.1}% {median_str:>12} {mean_naive:>12.1} {diff_str:>12}"
            );

            km_curves.push((*value, km));
        }
    }

    println!("  (Showing P0, P25, P50, P75, P100 by board count, total values: {total_values})");

    // Save CSV files if output directory specified
    if let Some(dir) = output_dir {
        std::fs::create_dir_all(dir)?;
        let csv_path = dir.join(format!("{feature_id}_km.csv"));
        let mut csv_content = String::from("value,time,survival_prob,at_risk,events\n");

        for (value, km) in &km_curves {
            for i in 0..km.times.len() {
                writeln!(
                    &mut csv_content,
                    "{},{},{},{},{}",
                    value, km.times[i], km.survival_prob[i], km.at_risk[i], km.events[i]
                )
                .unwrap();
            }
        }

        std::fs::write(&csv_path, csv_content)?;
        println!("\n  KM curves saved to: {}", csv_path.display());
    }

    Ok(())
}

#[expect(clippy::cast_precision_loss)]
fn analyze_by_evaluator(sessions: &[crate::data::SessionData]) {
    println!("Censoring by Evaluator:");
    println!(
        "  {:<15} {:>10} {:>10} {:>12}",
        "Evaluator", "Sessions", "Censored%", "Mean Surviv."
    );
    println!("  {}", "-".repeat(50));

    let mut evaluator_stats: HashMap<String, (Vec<usize>, usize)> = HashMap::new();

    for session in sessions {
        let is_censored = !session.is_game_over;
        let game_end = session.survived_turns;

        let entry = evaluator_stats
            .entry(session.placement_evaluator.clone())
            .or_insert_with(|| (vec![], 0));

        entry.0.push(game_end);
        if is_censored {
            entry.1 += 1;
        }
    }

    let mut evaluators: Vec<_> = evaluator_stats.iter().collect();
    evaluators.sort_by_key(|(name, _)| *name);

    for (evaluator, (survived_turns, censored_count)) in evaluators {
        let total = survived_turns.len();
        let censoring_rate = 100.0 * *censored_count as f64 / total as f64;
        let mean_survival = survived_turns.iter().sum::<usize>() as f64 / total as f64;

        println!("  {evaluator:<15} {total:>10} {censoring_rate:>9.1}% {mean_survival:>12.1}");
    }
}

#[expect(clippy::cast_precision_loss)]
fn analyze_by_feature(
    sessions: &[crate::data::SessionData],
    feature_id: &str,
) -> anyhow::Result<()> {
    // Find the feature
    let feature = ALL_BOARD_FEATURES
        .iter()
        .find(|f| f.id() == feature_id)
        .ok_or_else(|| anyhow::anyhow!("Feature {feature_id} not found"))?;

    println!("Feature: {} ({})", feature.name(), feature_id);

    // Collect data: feature_value -> [(remaining_turns, is_censored)]
    let mut feature_data: BTreeMap<u32, Vec<(usize, bool)>> = BTreeMap::new();

    for session in sessions {
        let is_censored = !session.is_game_over;
        let game_end = session.survived_turns;

        for board in &session.boards {
            // Compute feature value
            let analysis = PlacementAnalysis::from_board(&board.board, board.placement);
            let raw_value = feature.extract_raw(&analysis);

            let remaining = game_end - board.turn;
            feature_data
                .entry(raw_value)
                .or_default()
                .push((remaining, is_censored));
        }
    }

    // Show percentiles based on board count
    println!(
        "  {:<8} {:>8} {:>10} {:>12} {:>12} {:>8}",
        "Value", "Boards", "Censored%", "Mean(Comp)", "Mean(All)", "Bias"
    );
    println!("  {}", "-".repeat(68));

    let total_values = feature_data.len();

    // Calculate cumulative board counts for percentile-based selection
    let all_values: Vec<_> = feature_data.iter().collect();
    let total_boards: usize = all_values.iter().map(|(_, data)| data.len()).sum();

    let mut cumulative_boards = 0;
    let mut percentile_indices = Vec::new();
    let percentiles = [0.0, 0.25, 0.5, 0.75, 1.0];
    let mut percentile_idx = 0;

    for (idx, (_, data)) in all_values.iter().enumerate() {
        cumulative_boards += data.len();
        let current_percentile = cumulative_boards as f64 / total_boards as f64;

        while percentile_idx < percentiles.len()
            && current_percentile >= percentiles[percentile_idx]
        {
            percentile_indices.push(idx);
            percentile_idx += 1;
        }
    }

    // Ensure we always have the last value
    if (percentile_indices.is_empty()
        || *percentile_indices.last().unwrap() != all_values.len() - 1)
        && !percentile_indices.contains(&(all_values.len() - 1))
    {
        percentile_indices.push(all_values.len() - 1);
    }

    // Deduplicate indices
    percentile_indices.sort_unstable();
    percentile_indices.dedup();

    for &idx in &percentile_indices {
        if let Some((value, data)) = all_values.get(idx) {
            print_feature_value_row(**value, data);
        }
    }

    println!("  (Showing P0, P25, P50, P75, P100 by board count, total values: {total_values})");

    Ok(())
}

#[expect(clippy::cast_precision_loss)]
fn print_feature_value_row(value: u32, data: &[(usize, bool)]) {
    let total = data.len();
    let censored = data.iter().filter(|(_, c)| *c).count();
    let censoring_rate = 100.0 * censored as f64 / total as f64;

    let complete_remaining: Vec<usize> =
        data.iter().filter(|(_, c)| !*c).map(|(r, _)| *r).collect();

    let all_remaining: Vec<usize> = data.iter().map(|(r, _)| *r).collect();

    let mean_complete = if complete_remaining.is_empty() {
        0.0
    } else {
        complete_remaining.iter().sum::<usize>() as f64 / complete_remaining.len() as f64
    };

    let mean_all = all_remaining.iter().sum::<usize>() as f64 / all_remaining.len() as f64;

    let bias = if complete_remaining.is_empty() {
        "N/A".to_string()
    } else {
        let b = mean_all / mean_complete;
        if b > 1.5 {
            format!("⚠{b:.2}x")
        } else {
            format!("{b:.2}x")
        }
    };

    println!(
        "  {value:<8} {total:>8} {censoring_rate:>9.1}% {mean_complete:>12.1} {mean_all:>12.1} {bias:>8}"
    );
}
