use std::{io::Write as _, path::PathBuf};

use anyhow::Context;
use oxidris_ai::{placement_evaluator::DumpPlacementEvaluator, turn_evaluator::TurnEvaluator};
use oxidris_engine::{GameField, GameStats};

use crate::{
    data::{BoardAndPlacement, BoardCollection},
    util::Output,
};

const MAX_TURNS: usize = 500;
const TURNS_HISTOGRAM_WIDTH: usize = 10;
const HEIGHT_HISTOGRAM_WIDTH: usize = 2;

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct GenerateBoardsArg {
    /// Number of boards to generate
    #[arg(long, default_value_t = 100000)]
    num_boards: usize,
    /// Output file path
    #[arg(long)]
    output: Option<PathBuf>,
}

pub(crate) fn run(arg: &GenerateBoardsArg) -> anyhow::Result<()> {
    let GenerateBoardsArg { num_boards, output } = arg;
    let placement_evaluator = DumpPlacementEvaluator;
    let turn_evaluator = TurnEvaluator::new(placement_evaluator);

    eprintln!("Generating boards for training data...");

    let mut total_games = 0;
    let mut captured_boards = vec![];
    let mut turns_histogram = [0; MAX_TURNS / TURNS_HISTOGRAM_WIDTH + 1];
    let mut height_histogram = [0; 10];

    while captured_boards.len() < *num_boards {
        total_games += 1;
        let mut captured_heights = [false; 4];
        let mut field = GameField::new();
        let mut stats = GameStats::new();
        while let Some(turn) = turn_evaluator.select_best_turn(&field) {
            let t = stats.completed_pieces();
            let capture_board = BoardAndPlacement {
                board: field.board().clone(),
                placement: turn.placement(),
            };
            let (_cleared_lines, result) = turn.apply(&mut field, &mut stats);
            if result.is_err() {
                break;
            }

            let mut do_capture = false;
            let h = field.board().max_height();

            // Fixed intervals to capture boards
            if matches!(t, 5 | 15 | 30 | 60) || (t >= 100 && t.is_multiple_of(50)) {
                do_capture = true;
            }

            for (i, th) in [5, 9, 12, 14].into_iter().enumerate() {
                if h >= th && !captured_heights[i] {
                    captured_heights[i] = true;
                    do_capture = true;
                }
            }

            if do_capture {
                turns_histogram[(t / TURNS_HISTOGRAM_WIDTH).min(turns_histogram.len() - 1)] += 1;
                height_histogram[(h / HEIGHT_HISTOGRAM_WIDTH).min(height_histogram.len() - 1)] += 1;
                captured_boards.push(capture_board);
            }

            if stats.completed_pieces() > MAX_TURNS {
                break;
            }
        }
    }

    let captured_boards = BoardCollection {
        boards: captured_boards,
    };

    eprintln!(
        "Captured {} boards from {} games.",
        captured_boards.boards.len(),
        total_games,
    );
    eprintln!();
    eprintln!("Turns histogram:");
    for (i, count) in turns_histogram.iter().enumerate() {
        let min = i * TURNS_HISTOGRAM_WIDTH;
        eprintln!("  {min:2}- : {count:5} {}", "#".repeat(count / 50));
    }
    eprintln!();
    eprintln!("Height histogram:");
    for (i, count) in height_histogram.iter().enumerate() {
        let min = i * HEIGHT_HISTOGRAM_WIDTH;
        eprintln!("  {min:2}- : {count:5} {}", "#".repeat(count / 50));
    }

    let mut writer = Output::from_output_path(output.clone())?;
    serde_json::to_writer(&mut writer, &captured_boards).with_context(|| {
        format!(
            "Failed to write captured boards to {}",
            writer.display_path()
        )
    })?;
    writer.flush().with_context(|| {
        format!(
            "Failed to write captured boards to {}",
            writer.display_path()
        )
    })?;

    Ok(())
}
