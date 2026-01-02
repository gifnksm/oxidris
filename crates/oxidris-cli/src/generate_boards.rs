use std::path::PathBuf;

use oxidris_ai::{
    board_feature::{self, DynBoardFeatureSource},
    placement_analysis::PlacementAnalysis,
    placement_evaluator::{FeatureBasedPlacementEvaluator, PlacementEvaluator},
    turn_evaluator::TurnEvaluator,
};
use oxidris_engine::{GameField, GameStats};
use rand::Rng as _;

use crate::{
    data::{BoardAndPlacement, SessionCollection, SessionData},
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

type BoxedPlacementEvaluator = Box<dyn PlacementEvaluator>;
#[derive(Debug, Clone)]
struct PlacementEvaluatorFactory {
    name: &'static str,
    factory: fn() -> BoxedPlacementEvaluator,
}

impl PlacementEvaluatorFactory {
    pub fn new(
        name: &'static str,
        factory: fn() -> BoxedPlacementEvaluator,
    ) -> PlacementEvaluatorFactory {
        PlacementEvaluatorFactory { name, factory }
    }
}

pub(crate) fn run(arg: &GenerateBoardsArg) -> anyhow::Result<()> {
    let GenerateBoardsArg { num_boards, output } = arg;
    let placement_evaluators: &[PlacementEvaluatorFactory] = &[
        PlacementEvaluatorFactory::new("dumb", DumbPlacementEvaluator::boxed),
        PlacementEvaluatorFactory::new("aggro", AggroPlacementEvaluator::boxed),
        PlacementEvaluatorFactory::new("balance", BalancePlacenemtnEvaluator::boxed),
    ];

    eprintln!("Generating boards for training data...");

    let mut rng = rand::rng();
    let mut total_games = 0;
    let mut captured_boards = 0;
    let mut captured_sessions = vec![];
    let mut evaluator_histogram = vec![0; placement_evaluators.len()];
    let mut turns_histogram = [0; MAX_TURNS / TURNS_HISTOGRAM_WIDTH + 1];
    let mut height_histogram = [0; 10];

    while captured_boards < *num_boards {
        total_games += 1;
        let mut captured_heights = [false; 6];
        let mut field = GameField::new();
        let mut stats = GameStats::new();
        let evaluator_index = rng.random_range(0..placement_evaluators.len());
        let placement_evaluator = &placement_evaluators[evaluator_index];
        let turn_evaluator = TurnEvaluator::new((placement_evaluator.factory)());
        let mut session_data = SessionData {
            gameover_turn: None,
            placement_evaluator: placement_evaluator.name.to_owned(),
            boards: vec![],
        };
        while let Some((turn, analysis)) = turn_evaluator.select_best_turn(&field) {
            let t = stats.completed_pieces();
            let capture_board = BoardAndPlacement {
                turn: t,
                board: field.board().clone(),
                placement: turn.placement(),
            };
            let (_cleared_lines, result) = turn.apply(&analysis, &mut field, &mut stats);
            if result.is_err() {
                session_data.gameover_turn = Some(t);
                break;
            }

            let mut do_capture = false;
            let h = field.board().max_height();

            // Fixed intervals to capture boards
            if matches!(t, 5 | 15 | 30 | 60) || (t >= 100 && t.is_multiple_of(50)) {
                do_capture = true;
            }

            for (i, th) in [5, 9, 12, 14, 16, 18].into_iter().enumerate() {
                if h >= th && !captured_heights[i] {
                    captured_heights[i] = true;
                    do_capture = true;
                }
            }

            if do_capture {
                captured_boards += 1;
                evaluator_histogram[evaluator_index] += 1;
                turns_histogram[(t / TURNS_HISTOGRAM_WIDTH).min(turns_histogram.len() - 1)] += 1;
                height_histogram[(h / HEIGHT_HISTOGRAM_WIDTH).min(height_histogram.len() - 1)] += 1;
                session_data.boards.push(capture_board);
            }

            if stats.completed_pieces() > MAX_TURNS {
                break;
            }
        }
        captured_sessions.push(session_data);
    }

    let collection = SessionCollection {
        total_boards: captured_boards,
        sessions: captured_sessions,
    };

    eprintln!("Captured {captured_boards} boards from {total_games} games.");
    eprintln!();
    eprintln!("Placement evaluator histogram:");
    for (i, evaluator) in placement_evaluators.iter().enumerate() {
        let count = evaluator_histogram[i];
        eprintln!(
            "  {:10} : {count:5} {}",
            evaluator.name,
            "#".repeat(count / 200)
        );
    }
    eprintln!();
    eprintln!("Turns histogram:");
    for (i, count) in turns_histogram.iter().enumerate() {
        let min = i * TURNS_HISTOGRAM_WIDTH;
        eprintln!("  {min:3}- : {count:5} {}", "#".repeat(count / 100));
    }
    eprintln!();
    eprintln!("Height histogram:");
    for (i, count) in height_histogram.iter().enumerate() {
        let min = i * HEIGHT_HISTOGRAM_WIDTH;
        eprintln!("  {min:2}- : {count:5} {}", "#".repeat(count / 200));
    }

    Output::save_json(&collection, output.clone())?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct DumbPlacementEvaluator {}

impl DumbPlacementEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn boxed() -> BoxedPlacementEvaluator {
        Box::new(Self::new())
    }
}

impl PlacementEvaluator for DumbPlacementEvaluator {
    #[inline]
    fn evaluate_placement(&self, analysis: &PlacementAnalysis) -> f32 {
        let max_height = analysis.board_analysis().max_height();
        let covered_holes = analysis.board_analysis().num_holes();
        -f32::from(max_height) - f32::from(covered_holes)
    }
}

#[derive(Debug, Clone)]
pub struct AggroPlacementEvaluator {
    evaluator: FeatureBasedPlacementEvaluator,
}

impl AggroPlacementEvaluator {
    pub fn new() -> Self {
        let pairs: &[(&'static dyn DynBoardFeatureSource, f32)] = &[
            (&board_feature::RowTransitionsPenalty, 0.1f32),
            (&board_feature::ColumnTransitionsPenalty, 0.1),
            (&board_feature::SurfaceBumpinessPenalty, 0.1),
            (&board_feature::SurfaceRoughnessPenalty, 0.1),
            (&board_feature::WellDepthPenalty, 0.1),
            (&board_feature::MaxHeightPenalty, 0.1),
            (&board_feature::TopOutRisk, 0.1),
            (&board_feature::LineClearBonus, 0.5),
            (&board_feature::IWellReward, 0.5),
        ];
        let features = pairs.iter().map(|(f, _)| *f).collect();
        let weights = pairs.iter().map(|(_, w)| *w).collect();
        Self {
            evaluator: FeatureBasedPlacementEvaluator::new(features, weights),
        }
    }

    pub fn boxed() -> BoxedPlacementEvaluator {
        Box::new(Self::new())
    }
}

impl PlacementEvaluator for AggroPlacementEvaluator {
    #[inline]
    fn evaluate_placement(&self, analysis: &PlacementAnalysis) -> f32 {
        self.evaluator.evaluate_placement(analysis)
    }
}

#[derive(Debug, Clone)]
pub struct BalancePlacenemtnEvaluator {
    evaluator: FeatureBasedPlacementEvaluator,
}

impl BalancePlacenemtnEvaluator {
    pub fn new() -> Self {
        // copy from defensive.json
        let pairs: &[(&'static dyn DynBoardFeatureSource, f32)] = &[
            (&board_feature::HolesPenalty, 0.213_184_65),
            (&board_feature::HoleDepthPenalty, 0.110_842_15),
            (&board_feature::RowTransitionsPenalty, 0.092_231_855),
            (&board_feature::ColumnTransitionsPenalty, 0.008_030_57),
            (&board_feature::SurfaceBumpinessPenalty, 0.036_550_764),
            (&board_feature::SurfaceRoughnessPenalty, 0.019_671_245),
            (&board_feature::WellDepthPenalty, 0.205_994_08),
            (&board_feature::DeepWellRisk, 0.023_119_722),
            (&board_feature::MaxHeightPenalty, 0.139_035_12),
            (&board_feature::CenterColumnsPenalty, 0.029_494_1),
            (&board_feature::CenterTopOutRisk, 0.021_646_535),
            (&board_feature::TopOutRisk, 0.062_471_554),
            (&board_feature::TotalHeightPenalty, 0.033_591_066),
            (&board_feature::LineClearBonus, 0.003_266_561_8),
            (&board_feature::IWellReward, 0.000_870_029_5),
        ];
        let features = pairs.iter().map(|(f, _)| *f).collect();
        let weights = pairs.iter().map(|(_, w)| *w).collect();
        Self {
            evaluator: FeatureBasedPlacementEvaluator::new(features, weights),
        }
    }

    pub fn boxed() -> BoxedPlacementEvaluator {
        Box::new(Self::new())
    }
}

impl PlacementEvaluator for BalancePlacenemtnEvaluator {
    #[inline]
    fn evaluate_placement(&self, analysis: &PlacementAnalysis) -> f32 {
        self.evaluator.evaluate_placement(analysis)
    }
}
