use std::{collections::HashMap, fmt, path::PathBuf};

use oxidris_ai::{
    board_feature::{self, DynBoardFeatureSource},
    placement_analysis::PlacementAnalysis,
    placement_evaluator::{FeatureBasedPlacementEvaluator, PlacementEvaluator},
    turn_evaluator::TurnEvaluator,
};
use oxidris_engine::{GameField, GameStats};
use rand::Rng;

use crate::{
    data::{BoardAndPlacement, SessionCollection, SessionData},
    util::Output,
};

const MAX_TURNS: usize = 500;
const TURNS_HISTOGRAM_WIDTH: usize = 10;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DifficultyBin {
    height_bin: u8,
    holes_bin: u8,
}

impl DifficultyBin {
    const NUM_BINS: usize = 20;
    const HEIGHT_BIN_WIDTH: u8 = 4;
    const HOLES_BIN_WIDTH: u8 = 3;

    fn new(analysis: &PlacementAnalysis) -> Self {
        let height = analysis.board_analysis().max_height();
        let holes = analysis.board_analysis().num_holes();
        Self {
            height_bin: (height / Self::HEIGHT_BIN_WIDTH).clamp(0, 4),
            holes_bin: (holes / Self::HOLES_BIN_WIDTH).clamp(0, 3),
        }
    }
}

struct AdaptiveSampler {
    bin_counts: HashMap<DifficultyBin, usize>,
    total_captured: usize,
}

impl AdaptiveSampler {
    fn new() -> Self {
        let bin_counts = (0..4)
            .flat_map(|height| {
                (0..4).map(move |holes| {
                    (
                        DifficultyBin {
                            height_bin: height,
                            holes_bin: holes,
                        },
                        0,
                    )
                })
            })
            .collect();
        Self {
            bin_counts,
            total_captured: 0,
        }
    }

    #[expect(clippy::cast_precision_loss)]
    fn should_capture<R>(
        &mut self,
        analysis: &PlacementAnalysis,
        stats: &GameStats,
        rng: &mut R,
    ) -> bool
    where
        R: Rng,
    {
        let bin = DifficultyBin::new(analysis);
        let current_count = *self.bin_counts.get(&bin).unwrap_or(&0);
        let desired_count = self.total_captured.div_ceil(DifficultyBin::NUM_BINS);
        let fill_ratio = if desired_count == 0 {
            0.0
        } else {
            current_count as f64 / desired_count as f64
        };

        let mut capture_prob = 1.0 / (1.0 + fill_ratio);

        let difficulty_score = f64::from(bin.height_bin + bin.holes_bin);
        let difficulty_bonus = (difficulty_score / 7.0).powf(2.5) * 3.0;
        // (0, 0) -> 0.0
        // (4, 3) -> (7/7 )^2.5 * 3 = 3.0
        capture_prob += difficulty_bonus;

        let turns = stats.completed_pieces();
        let turn_multiplier = if turns < 30 {
            0.5
        } else if turns < 100 {
            1.0
        } else {
            1.2
        };
        capture_prob *= turn_multiplier;

        // Downscale capture probability to avoid excessive captures
        capture_prob *= 0.3;
        capture_prob = capture_prob.clamp(0.05, 1.0);

        let should_capture = rng.random_bool(capture_prob);
        if should_capture {
            *self.bin_counts.entry(bin).or_insert(0) += 1;
            self.total_captured += 1;
        }
        should_capture
    }

    fn print_progress(&self) {
        eprintln!("Captured {} boards", self.total_captured);
        eprintln!("Height & Holes distribution:");

        let mut bins: Vec<_> = self.bin_counts.iter().collect();
        bins.sort_by_key(|(bin, _)| (bin.height_bin, bin.holes_bin));

        print_histogram(bins.into_iter().map(|(bin, count)| {
            let height = bin.height_bin * DifficultyBin::HEIGHT_BIN_WIDTH;
            let holes = bin.holes_bin * DifficultyBin::HOLES_BIN_WIDTH;
            (format!("{height:2},{holes:2}"), *count)
        }));
    }
}

const CAPTURE_INTERVAL: usize = 5;

pub(crate) fn run(arg: &GenerateBoardsArg) -> anyhow::Result<()> {
    let GenerateBoardsArg { num_boards, output } = arg;
    let placement_evaluators: &[PlacementEvaluatorFactory] = &[
        PlacementEvaluatorFactory::new("dumb", DumbPlacementEvaluator::boxed),
        PlacementEvaluatorFactory::new("aggro", AggroPlacementEvaluator::boxed),
        PlacementEvaluatorFactory::new("balance", BalancePlacementEvaluator::boxed),
    ];

    eprintln!("Generating boards for training data...");

    let mut rng = rand::rng();
    let mut total_games = 0;
    let mut captured_sessions = vec![];
    let mut evaluator_histogram = vec![0; placement_evaluators.len()];
    let mut turns_histogram = [0; MAX_TURNS / TURNS_HISTOGRAM_WIDTH + 1];
    let mut adaptive_sampler = AdaptiveSampler::new();

    while adaptive_sampler.total_captured < *num_boards {
        total_games += 1;
        let mut field = GameField::new();
        let mut stats = GameStats::new();
        let evaluator_index = rng.random_range(0..placement_evaluators.len());
        let placement_evaluator = &placement_evaluators[evaluator_index];
        let turn_evaluator = TurnEvaluator::new((placement_evaluator.factory)());
        let mut session_data = SessionData {
            placement_evaluator: placement_evaluator.name.to_owned(),
            survived_turns: 0,
            is_game_over: false,
            boards: vec![],
        };
        let mut capture_interval = CAPTURE_INTERVAL;
        while let Some((turn_plan, analysis)) = turn_evaluator.select_best_turn(&field) {
            let turn = stats.completed_pieces();
            let capture_board = BoardAndPlacement {
                turn,
                board: field.board().clone(),
                placement: turn_plan.placement(),
            };
            let (_cleared_lines, result) = turn_plan.apply(&analysis, &mut field, &mut stats);
            if result.is_err() {
                session_data.is_game_over = true;
                break;
            }
            session_data.survived_turns = turn;

            if capture_interval == 0 {
                if adaptive_sampler.should_capture(&analysis, &stats, &mut rng) {
                    evaluator_histogram[evaluator_index] += 1;
                    turns_histogram
                        [(turn / TURNS_HISTOGRAM_WIDTH).min(turns_histogram.len() - 1)] += 1;
                    session_data.boards.push(capture_board);
                    capture_interval = CAPTURE_INTERVAL;
                }
                if adaptive_sampler.total_captured.is_multiple_of(1000)
                    && adaptive_sampler.total_captured > 0
                {
                    adaptive_sampler.print_progress();
                }
            } else {
                capture_interval -= 1;
            }
            if stats.completed_pieces() > MAX_TURNS {
                break;
            }
        }
        captured_sessions.push(session_data);
    }

    let collection = SessionCollection {
        total_boards: adaptive_sampler.total_captured,
        sessions: captured_sessions,
    };

    eprintln!(
        "Captured {} boards from {total_games} games.",
        adaptive_sampler.total_captured,
    );
    adaptive_sampler.print_progress();
    eprintln!();
    eprintln!("Placement evaluator histogram:");
    print_histogram(
        placement_evaluators
            .iter()
            .enumerate()
            .map(|(i, evaluator)| (evaluator.name, evaluator_histogram[i])),
    );
    eprintln!();
    eprintln!("Turns histogram:");
    print_histogram(
        turns_histogram
            .iter()
            .enumerate()
            .map(|(i, count)| (i * TURNS_HISTOGRAM_WIDTH, *count)),
    );

    Output::save_json(&collection, output.clone())?;

    Ok(())
}

fn print_histogram<I, S>(data: I)
where
    I: Iterator<Item = (S, usize)>,
    S: fmt::Display,
{
    let data = data.collect::<Vec<_>>();
    let max_count = data.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let max_bar_width = 50;
    for (label, count) in &data {
        let bar_width = (count * max_bar_width) / max_count;
        println!("{:>10} | {:<5} {}", label, count, "#".repeat(bar_width));
    }
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
        // copy from aggro.json
        let pairs: &[(&'static dyn DynBoardFeatureSource, f32)] = &[
            (&board_feature::HolesPenalty, 0.228_862_03),
            (&board_feature::HoleDepthPenalty, 0.157_612_1),
            (&board_feature::RowTransitionsPenalty, 0.032_375_332),
            (&board_feature::ColumnTransitionsPenalty, 0.073_017_87),
            (&board_feature::SurfaceBumpinessPenalty, 0.021_512_082),
            (&board_feature::SurfaceRoughnessPenalty, 0.0),
            (&board_feature::WellDepthPenalty, 0.003_433_691_3),
            (&board_feature::DeepWellRisk, 0.112_219_19),
            (&board_feature::MaxHeightPenalty, 0.041_565_202),
            (&board_feature::CenterColumnsPenalty, 0.017_524_77),
            (&board_feature::CenterTopOutRisk, 0.006_517_862),
            (&board_feature::TopOutRisk, 0.238_829_42),
            (&board_feature::TotalHeightPenalty, 0.021_264_264),
            (&board_feature::LineClearBonus, 0.010_285_562),
            (&board_feature::IWellReward, 0.034_980_606),
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
pub struct BalancePlacementEvaluator {
    evaluator: FeatureBasedPlacementEvaluator,
}

impl BalancePlacementEvaluator {
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

impl PlacementEvaluator for BalancePlacementEvaluator {
    #[inline]
    fn evaluate_placement(&self, analysis: &PlacementAnalysis) -> f32 {
        self.evaluator.evaluate_placement(analysis)
    }
}
