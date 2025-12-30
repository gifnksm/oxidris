use std::iter;

use oxidris_ai::{
    AiType,
    board_feature::ALL_BOARD_FEATURES,
    genetic::{Population, PopulationEvolver},
    session_evaluator::{AggroSessionEvaluator, DefensiveSessionEvaluator, SessionEvaluator},
    statistics,
};
use oxidris_engine::GameField;

use crate::util;

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
    #[arg(long, default_value = "aggro")]
    ai: AiType,
}

#[expect(clippy::cast_precision_loss)]
pub(crate) fn run(arg: &TrainAiArg) {
    let TrainAiArg { ai } = arg;
    let session_evaluator = match ai {
        AiType::Aggro => &AggroSessionEvaluator::new(TURN_LIMIT) as &dyn SessionEvaluator,
        AiType::Defensive => &DefensiveSessionEvaluator::new(TURN_LIMIT) as &dyn SessionEvaluator,
    };
    let board_features = &ALL_BOARD_FEATURES;

    let mut rng = rand::rng();
    let mut population = Population::random(
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
        population.evaluate_fitness(&fields, session_evaluator, board_features);

        let weight_stats = population.compute_weight_stats();
        let weight_norm_std_dev_mean = {
            let norm_std_devs = weight_stats.iter().map(|s| s.normalized_std_dev);
            statistics::compute_mean(norm_std_devs).unwrap()
        };

        let fitness_stats = population.compute_fitness_stats();

        eprintln!("  Individuals:");
        for (i, ind) in population.individuals().iter().enumerate() {
            eprintln!(
                "  {i:2}: {:.3?} => {:.3}",
                ind.weights().as_array(),
                ind.fitness()
            );
        }

        eprintln!("  Weights Stats:");
        eprintln!(
            "    Min:        {:.3?}",
            weight_stats.each_ref().map(|s| s.min)
        );
        eprintln!(
            "    Max:        {:.3?}",
            weight_stats.each_ref().map(|s| s.max)
        );
        eprintln!(
            "    Mean:       {:.3?}",
            weight_stats.each_ref().map(|s| s.mean)
        );
        eprintln!(
            "    NormStddev: {:.3?}",
            weight_stats.each_ref().map(|s| s.normalized_std_dev)
        );
        eprintln!("    => Mean:    {weight_norm_std_dev_mean:.3}");

        eprintln!("  Fitness Stats:");
        eprintln!("    Min:  {:.3}", fitness_stats.min);
        eprintln!("    Max:  {:.3}", fitness_stats.max);
        eprintln!("    Mean: {:.3}", fitness_stats.mean);

        if generation + 1 < MAX_GENERATIONS {
            population = evolver.evolve(&population);
        }
    }

    eprintln!("Best Individuals:");
    for (i, ind) in population.individuals().iter().take(5).enumerate() {
        eprintln!(
            "  {i:2}: {:?} => {}",
            ind.weights().as_array(),
            ind.fitness()
        );
    }

    eprintln!("Best individual weights saved to file:");
    let best_individual = population.individuals().first().unwrap();
    eprintln!("[");
    for (f, w) in iter::zip(
        ALL_BOARD_FEATURES.as_array(),
        best_individual.weights().as_array(),
    ) {
        let scale = w / (1.0 / ALL_BOARD_FEATURES.len() as f32);
        eprintln!("    {}, // {} (x{scale:.3})", util::format_f32(w), f.name());
    }
    eprintln!("]");

    eprintln!("{ai:?} AI learning completed.");
}
