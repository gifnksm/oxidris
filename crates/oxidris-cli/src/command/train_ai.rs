use std::{iter, path::PathBuf};

use chrono::Utc;
use oxidris_engine::GameField;
use oxidris_evaluator::{
    board_feature::BoxedBoardFeature,
    session_evaluator::{
        AggroSessionEvaluator, DefaultSessionEvaluator, DefensiveSessionEvaluator, SessionEvaluator,
    },
};
use oxidris_training::genetic::{Individual, Population, PopulationEvolver};

use crate::{
    model::ai_model::{AiModel, TrainedBoardFeature},
    util::{self, FeatureSet, Output},
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, derive_more::FromStr)]
#[from_str(rename_all = "kebab-case")]
pub enum AiType {
    #[default]
    AggroKm,
    DefensiveKm,
    AggroRaw,
    DefensiveRaw,
}

const GAMES_PER_INDIVIDUAL: usize = 3;
const TURN_LIMIT: usize = 3000;

const POPULATION_COUNT: usize = 30;

const MAX_GENERATIONS: usize = 200;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum EvolutaionPhase {
    #[default]
    Exploration,
    Transition,
    Convergence,
}

impl EvolutaionPhase {
    fn from_generation(generation: usize) -> Self {
        match generation {
            0..30 => Self::Exploration,
            30..80 => Self::Transition,
            _ => Self::Convergence,
        }
    }
}
const ELITE_COUNT: usize = 2;
const TOURNAMENT_SIZE: usize = 2;

const fn max_weight_by_phase(phase: EvolutaionPhase) -> f32 {
    match phase {
        EvolutaionPhase::Exploration => 0.5,
        EvolutaionPhase::Transition => 0.8,
        EvolutaionPhase::Convergence => 1.0,
    }
}

const MUTATION_RATE: f32 = 0.3;
const fn mutation_sigma_by_phase(phase: EvolutaionPhase) -> f32 {
    match phase {
        EvolutaionPhase::Exploration => 0.05,
        EvolutaionPhase::Transition => 0.02,
        EvolutaionPhase::Convergence => 0.01,
    }
}

const BLX_ALPHA: f32 = 0.2;

const fn evolver_by_phase(phase: EvolutaionPhase) -> PopulationEvolver {
    PopulationEvolver {
        elite_count: ELITE_COUNT,
        tournament_size: TOURNAMENT_SIZE,
        max_weight: max_weight_by_phase(phase),
        mutation_sigma: mutation_sigma_by_phase(phase),
        blx_alpha: BLX_ALPHA,
        mutation_rate: MUTATION_RATE,
    }
}

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct TrainAiArg {
    /// Boards data file path
    boards_file: PathBuf,
    #[arg(long, default_value = "aggro")]
    ai: AiType,
    /// Output file path
    #[arg(long)]
    output: Option<PathBuf>,
}

pub(crate) fn run(arg: &TrainAiArg) -> anyhow::Result<()> {
    let TrainAiArg {
        boards_file,
        ai,
        output,
    } = arg;

    eprintln!("Loading boards from {}...", boards_file.display());
    let sessions = util::read_boards_file(boards_file)?.sessions;
    eprintln!("Loaded {} sessions", sessions.len());

    let (session_evaluator, feature_set) = match ai {
        AiType::AggroKm => (
            &DefaultSessionEvaluator::new(TURN_LIMIT, AggroSessionEvaluator::new())
                as &dyn SessionEvaluator,
            FeatureSet::Km,
        ),
        AiType::DefensiveKm => (
            &DefaultSessionEvaluator::new(TURN_LIMIT, DefensiveSessionEvaluator::new())
                as &dyn SessionEvaluator,
            FeatureSet::Km,
        ),
        AiType::AggroRaw => (
            &DefaultSessionEvaluator::new(TURN_LIMIT, AggroSessionEvaluator::new())
                as &dyn SessionEvaluator,
            FeatureSet::Raw,
        ),
        AiType::DefensiveRaw => (
            &DefaultSessionEvaluator::new(TURN_LIMIT, DefensiveSessionEvaluator::new())
                as &dyn SessionEvaluator,
            FeatureSet::Raw,
        ),
    };

    let features = util::build_feature_from_session(feature_set, &sessions)?;

    let mut rng = rand::rng();
    let mut population = Population::random(
        features.clone(),
        POPULATION_COUNT,
        &mut rng,
        max_weight_by_phase(EvolutaionPhase::default()),
    );
    for generation in 0..MAX_GENERATIONS {
        let phase = EvolutaionPhase::from_generation(generation);
        eprintln!("Generation #{generation} ({phase:?}):");
        let evolver = evolver_by_phase(phase);
        let fields: Vec<GameField> = (0..GAMES_PER_INDIVIDUAL)
            .map(|_| GameField::new())
            .collect();
        population.evaluate_fitness(&fields, session_evaluator);

        print_generation_summary(&population);

        if generation + 1 < MAX_GENERATIONS {
            population = evolver.evolve(&population);
        }
    }

    print_best_individuals(&population);
    eprintln!("{ai:?} AI learning completed.");

    let model_name = match ai {
        AiType::AggroKm => "aggro-km",
        AiType::DefensiveKm => "defensive-km",
        AiType::AggroRaw => "aggro-raw",
        AiType::DefensiveRaw => "defensive-raw",
    };
    let best_individual = population.individuals().first().unwrap();
    save_model(output.as_ref(), model_name, &features, best_individual)?;

    Ok(())
}

fn print_generation_summary(population: &Population) {
    let weight_stats = population.compute_weight_stats();
    #[expect(clippy::cast_precision_loss)]
    let weight_norm_std_dev_mean = weight_stats
        .iter()
        .map(|s| s.normalized_std_dev)
        .sum::<f32>()
        / weight_stats.len() as f32;

    let fitness_stats = population.compute_fitness_stats();

    eprintln!("  Individuals:");
    for (i, ind) in population.individuals().iter().enumerate() {
        eprintln!("  {i:2}: {:.3?} => {:.3}", ind.weights(), ind.fitness());
    }

    eprintln!("  Weights Stats:");
    eprintln!(
        "    Min:        {:.3?}",
        weight_stats.iter().map(|s| s.min).collect::<Vec<_>>(),
    );
    eprintln!(
        "    Max:        {:.3?}",
        weight_stats.iter().map(|s| s.max).collect::<Vec<_>>(),
    );
    eprintln!(
        "    Mean:       {:.3?}",
        weight_stats.iter().map(|s| s.mean).collect::<Vec<_>>(),
    );
    eprintln!(
        "    NormStddev: {:.3?}",
        weight_stats
            .iter()
            .map(|s| s.normalized_std_dev)
            .collect::<Vec<_>>(),
    );
    eprintln!("    => Mean:    {weight_norm_std_dev_mean:.3}");

    eprintln!("  Fitness Stats:");
    eprintln!("    Min:  {:.3}", fitness_stats.min);
    eprintln!("    Max:  {:.3}", fitness_stats.max);
    eprintln!("    Mean: {:.3}", fitness_stats.mean);
}

fn print_best_individuals(population: &Population) {
    eprintln!("Best Individuals:");
    for (i, ind) in population.individuals().iter().take(5).enumerate() {
        eprintln!("  {i:2}: {:?} => {}", ind.weights(), ind.fitness());
    }
}

fn save_model(
    output_path: Option<&PathBuf>,
    model_name: &str,
    features: &[BoxedBoardFeature],
    individual: &Individual,
) -> anyhow::Result<()> {
    let model = AiModel {
        name: model_name.to_owned(),
        trained_at: Utc::now(),
        final_fitness: individual.fitness(),
        board_features: iter::zip(features, individual.weights())
            .map(|(f, w)| {
                let source = f.feature_source();
                TrainedBoardFeature {
                    id: f.id().to_owned(),
                    name: f.name().to_owned(),
                    source_id: source.id().to_owned(),
                    processing: f.feature_processing(),
                    weight: *w,
                }
            })
            .collect(),
    };
    Output::save_json(&model, output_path.cloned())?;

    eprintln!();
    eprintln!("Model saved successfully");
    if let Some(path) = output_path {
        eprintln!("  Path: {}", path.display());
    }
    eprintln!("  Name: {}", model.name);
    eprintln!("  Trained at: {}", model.trained_at);
    eprintln!("  Final fitness: {:.3}", model.final_fitness);
    eprintln!("  Weights: {} features", model.board_features.len());

    Ok(())
}
