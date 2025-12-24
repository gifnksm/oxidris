use oxidris_ai::{DumpPlacementEvaluator, TurnEvaluator};
use oxidris_engine::{GameField, GameStats};

pub(crate) fn run() {
    let placement_evaluator = DumpPlacementEvaluator;
    let turn_evaluator = TurnEvaluator::new(placement_evaluator);

    eprintln!("Generating boards for training data...");

    let mut total_games = 0;
    let mut captured_boards = vec![];
    let mut turns_histogram = [0; 10];
    let mut height_histogram = [0; 10];

    while captured_boards.len() < 10000 {
        total_games += 1;
        let mut captured_heights = [false; 4];
        let mut field = GameField::new();
        let mut stats = GameStats::new();
        while let Some(turn) = turn_evaluator.select_best_turn(&field) {
            let t = stats.completed_pieces();
            let (_cleared_lines, result) = turn.apply(&mut field, &mut stats);
            if result.is_err() {
                break;
            }

            let h = field.board().max_height();

            // Fixed intervals to capture boards
            if matches!(t, 5 | 15 | 30 | 60) {
                turns_histogram[(t / 10).min(9)] += 1;
                height_histogram[(h / 2).min(9)] += 1;
                captured_boards.push(field.board().clone());
            }

            for (i, th) in [5, 9, 12, 14].into_iter().enumerate() {
                if h >= th && !captured_heights[i] {
                    captured_heights[i] = true;
                    turns_histogram[(t / 10).min(9)] += 1;
                    height_histogram[(h / 2).min(9)] += 1;
                    captured_boards.push(field.board().clone());
                }
            }
            if h >= 15 {
                break;
            }
            if stats.completed_pieces() >= 100 {
                break;
            }
        }
    }

    serde_json::to_writer(std::io::stdout().lock(), &captured_boards).unwrap();

    eprintln!(
        "Captured {} boards from {} games.",
        captured_boards.len(),
        total_games,
    );
    eprintln!();
    eprintln!("Turns histogram:");
    for (i, count) in turns_histogram.iter().enumerate() {
        let min = i * 10;
        eprintln!("  {min:2}- : {count:5} {}", "#".repeat(count / 50));
    }
    eprintln!();
    eprintln!("Height histogram:");
    for (i, count) in height_histogram.iter().enumerate() {
        let min = i * 2;
        eprintln!("  {min:2}- : {count:5} {}", "#".repeat(count / 50));
    }
}
