use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use clap::Args;
use oxidris_ai::{board_feature::ALL_BOARD_FEATURES, placement_analysis::PlacementAnalysis};

use crate::data::load_sessions;

const MAX_TURNS: usize = 500;

#[derive(Debug, Clone, Args)]
pub(crate) struct AnalyzeCensoringArg {
    /// Path to the boards JSON file
    pub boards: PathBuf,

    /// Show detailed feature value breakdown
    #[arg(long)]
    pub detailed: bool,

    /// Feature IDs to analyze
    #[arg(long, default_values = ["holes_penalty", "max_height_penalty", "hole_depth_penalty"])]
    pub features: Vec<String>,
}

pub(crate) fn run(arg: &AnalyzeCensoringArg) -> anyhow::Result<()> {
    let sessions = load_sessions(&arg.boards)?;

    println!("Censoring Analysis Report (MAX_TURNS={MAX_TURNS})");
    println!("==========================================\n");

    analyze_overall_censoring(&sessions);
    println!();

    analyze_by_capture_phase(&sessions);
    println!();

    analyze_by_evaluator(&sessions);
    println!();

    for feature_id in &arg.features {
        analyze_by_feature(&sessions, feature_id, arg.detailed)?;
        println!();
    }

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
fn analyze_by_capture_phase(sessions: &[crate::data::SessionData]) {
    println!("Censoring by Capture Phase:");
    println!(
        "  {:<20} {:>8} {:>10} {:>12} {:>12} {:>8}",
        "Phase", "Boards", "Censored%", "Mean(Comp)", "Mean(All)", "Bias"
    );
    println!("  {}", "-".repeat(78));

    // Define phases dynamically based on MAX_TURNS
    let phase_size = MAX_TURNS / 4;
    let phases = [
        ("Early", 0, phase_size),
        ("Mid", phase_size, phase_size * 2),
        ("Late", phase_size * 2, phase_size * 3),
        ("Very Late", phase_size * 3, MAX_TURNS),
    ];

    for (phase_name, start, end) in phases {
        let mut phase_boards = Vec::new();

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
            .or_insert_with(|| (Vec::new(), 0));

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

fn analyze_by_feature(
    sessions: &[crate::data::SessionData],
    feature_id: &str,
    detailed: bool,
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

    if detailed {
        // Detailed output: show all values
        println!(
            "  {:<8} {:>8} {:>10} {:>12} {:>12} {:>8}",
            "Value", "Boards", "Censored%", "Mean(Comp)", "Mean(All)", "Bias"
        );
        println!("  {}", "-".repeat(68));

        for (value, data) in feature_data.iter().take(15) {
            print_feature_value_row(*value, data);
        }

        let total_values = feature_data.len();
        if total_values > 15 {
            println!("  ... and {} more values", total_values - 15);
        }
    } else {
        // Summary output: show quartiles
        println!(
            "  {:<8} {:>8} {:>10} {:>12} {:>12} {:>8}",
            "Value", "Boards", "Censored%", "Mean(Comp)", "Mean(All)", "Bias"
        );
        println!("  {}", "-".repeat(68));

        let total_values = feature_data.len();
        let quartile_indices = if total_values <= 5 {
            // Show all if few values
            (0..total_values).collect::<Vec<_>>()
        } else {
            // Show min, Q1, median, Q3, max
            vec![
                0,
                total_values / 4,
                total_values / 2,
                3 * total_values / 4,
                total_values - 1,
            ]
        };

        let all_values: Vec<_> = feature_data.iter().collect();
        for &idx in &quartile_indices {
            if let Some((value, data)) = all_values.get(idx) {
                print_feature_value_row(**value, data);
            }
        }

        if total_values > 5 {
            println!("  (Showing min, Q1, median, Q3, max of {total_values} values)");
        }
    }

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
