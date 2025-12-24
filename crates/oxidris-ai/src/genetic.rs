use std::{array, iter, thread};

use arrayvec::ArrayVec;
use rand::{
    Rng,
    distr::{Distribution, StandardUniform},
    seq::IndexedRandom,
};

use oxidris_engine::GameState;

use crate::{AiType, metrics::METRIC_COUNT, turn_evaluator::TurnEvaluator, weights::WeightSet};

use super::metrics::HeightInfo;

const GAMES_PER_INDIVIDUALS: usize = 3;
const MAX_PIECES_PER_GAME: usize = 800;

const POPULATION_COUNT: usize = 30;

const ELITE_COUNT: usize = 2;
const TOURNAMENT_SIZE: usize = 2;

// evolution parameters
const MAX_GENERATIONS: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvolutaionPhase {
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

const fn max_weight_by_phase(phase: EvolutaionPhase) -> f32 {
    match phase {
        EvolutaionPhase::Exploration => 0.5,
        EvolutaionPhase::Transition => 0.8,
        EvolutaionPhase::Convergence => 1.0,
    }
}

const MUTATION_RATE: f64 = 0.3;
const fn mutation_sigma_by_phase(phase: EvolutaionPhase) -> f32 {
    match phase {
        EvolutaionPhase::Exploration => 0.05,
        EvolutaionPhase::Transition => 0.02,
        EvolutaionPhase::Convergence => 0.01,
    }
}

const BLX_ALPHA: f32 = 0.2;

#[derive(Debug, Clone)]
struct Individual {
    weights: WeightSet<METRIC_COUNT>,
    fitness: f32,
}

impl Individual {
    #[expect(clippy::cast_precision_loss)]
    fn evaluate(&mut self, games: &[GameState], fitness_evaluator: &dyn FitnessEvaluator) {
        let turn_evaluator = TurnEvaluator::new(self.weights.clone());
        self.fitness = 0.0;
        for mut game in games.iter().cloned() {
            for _ in 0..MAX_PIECES_PER_GAME {
                let Some((_, next_game)) = turn_evaluator.select_best_turn(&game) else {
                    break;
                };
                game = next_game;
            }

            self.fitness += fitness_evaluator.evaluate(&game);
        }
        self.fitness /= games.len() as f32;
    }
}

trait FitnessEvaluator: Send + Sync {
    fn evaluate(&self, game: &GameState) -> f32;
}

#[derive(Debug)]
struct AggroFitnessEvaluator;

impl FitnessEvaluator for AggroFitnessEvaluator {
    #[expect(clippy::cast_precision_loss)]
    fn evaluate(&self, game: &GameState) -> f32 {
        const LINE_CLEAR_WEIGHT: [u16; 5] = [0, 1, 3, 5, 8];

        let survived = game.completed_pieces() as f32;
        let max_pieces = MAX_PIECES_PER_GAME as f32;
        let survived_ratio = survived / max_pieces;
        let survival_bonus = 2.0 * survived_ratio * survived_ratio;
        let weighted_line_count = iter::zip(LINE_CLEAR_WEIGHT, game.line_cleared_counter())
            .map(|(w, c)| f32::from(w) * (*c as f32))
            .sum::<f32>();
        let efficiency = weighted_line_count / survived.max(1.0);
        survival_bonus + efficiency * survived_ratio
    }
}

#[derive(Debug)]
struct DefensiveFitnessEvaluator;

impl FitnessEvaluator for DefensiveFitnessEvaluator {
    #[expect(clippy::cast_precision_loss)]
    fn evaluate(&self, game: &GameState) -> f32 {
        let height_info = HeightInfo::compute(game.board());
        let survived = game.completed_pieces() as f32;
        let max_pieces = MAX_PIECES_PER_GAME as f32;
        let survived_ratio = survived / max_pieces;
        let survival_bonus = 2.0 * survived_ratio * survived_ratio;
        let line_count = game.total_cleared_lines() as f32;
        let efficiency = line_count / survived.max(1.0);
        let height_penalty = f32::from(height_info.max_height()) / 20.0;
        survival_bonus + efficiency * survived_ratio - height_penalty
    }
}

#[expect(clippy::cast_precision_loss)]
pub fn learning(ai: AiType) {
    let fitness_evaluator = match ai {
        AiType::Aggro => &AggroFitnessEvaluator as &dyn FitnessEvaluator,
        AiType::Defensive => &DefensiveFitnessEvaluator as &dyn FitnessEvaluator,
    };
    let mut rng = rand::rng();
    let mut population = gen_first_generation(&mut rng);
    for generation in 0..MAX_GENERATIONS {
        let phase = EvolutaionPhase::from_generation(generation);
        println!("Generation #{generation} ({phase:?}):");
        let games: &[_; GAMES_PER_INDIVIDUALS] = &array::from_fn(|_| GameState::new());
        thread::scope(|s| {
            for (i, ind) in population.iter_mut().enumerate() {
                s.spawn(move || {
                    ind.evaluate(games, fitness_evaluator);
                    println!("  {i:2}: {:.3?} => {:.3}", ind.weights, ind.fitness);
                });
            }
        });

        let population_count = POPULATION_COUNT as f32;

        let weights = |i| population.iter().map(move |ind| ind.weights.to_array()[i]);
        let weights_min = WeightSet::<METRIC_COUNT>::from_fn(|i| min(weights(i)));
        let weights_max = WeightSet::<METRIC_COUNT>::from_fn(|i| max(weights(i)));
        let weights_mean = WeightSet::<METRIC_COUNT>::from_fn(|i| mean(weights(i)));
        let weights_norm_stddev =
            WeightSet::<METRIC_COUNT>::from_fn(|i| normalized_stddev(weights(i)));
        let weights_norm_stddev_mean = mean(weights_norm_stddev.to_array());

        let fitness_mean = population.iter().map(|i| i.fitness).sum::<f32>() / population_count;
        let fitness_max = population
            .iter()
            .map(|i| i.fitness)
            .max_by(f32::total_cmp)
            .unwrap();
        let fitness_min = population
            .iter()
            .map(|i| i.fitness)
            .min_by(f32::total_cmp)
            .unwrap();

        println!("  Weights Stats:");
        println!("    Min:        {weights_min:.3?}");
        println!("    Max:        {weights_max:.3?}");
        println!("    Mean:       {weights_mean:.3?}");
        println!("    NormStddev: {weights_norm_stddev:.3?}");
        println!("    => Mean:    {weights_norm_stddev_mean:.3}");
        println!("  Fitness Stats:");
        println!("    Min:  {fitness_min:.3}");
        println!("    Max:  {fitness_max:.3}");
        println!("    Mean: {fitness_mean:.3}");

        if generation + 1 < MAX_GENERATIONS {
            gen_next_generation(&mut population, phase, &mut rng);
        }
    }

    println!("Best Individuals:");
    population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    for (i, ind) in population.iter().take(5).enumerate() {
        println!("  {i:2}: {:?} => {}", ind.weights.to_array(), ind.fitness);
    }
}

fn mean(values: impl IntoIterator<Item = f32>) -> f32 {
    let (sum, count) = values
        .into_iter()
        .fold((0.0, 0.0), |(sum, count), x| (sum + x, count + 1.0));
    if count == 0.0 {
        return f32::NAN;
    }
    sum / count
}

fn max(values: impl IntoIterator<Item = f32>) -> f32 {
    values.into_iter().max_by(f32::total_cmp).unwrap()
}

fn min(values: impl IntoIterator<Item = f32>) -> f32 {
    values.into_iter().min_by(f32::total_cmp).unwrap()
}

fn normalized_stddev(values: impl IntoIterator<Item = f32> + Clone) -> f32 {
    let m = mean(values.clone());
    let variance = mean(values.clone().into_iter().map(|x| (x - m).powi(2)));
    let stddev = variance.sqrt();

    let max_v = max(values.clone());
    let min_v = min(values);

    let range = max_v - min_v;
    if range.abs() < f32::EPSILON {
        return 0.0;
    }
    stddev / range
}

impl Distribution<Individual> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Individual {
        let max_w = max_weight_by_phase(EvolutaionPhase::Exploration);
        let mut weights = WeightSet::random(rng, max_w);
        weights.normalize_l1();
        Individual {
            weights,
            fitness: f32::MIN,
        }
    }
}

fn gen_first_generation<R>(rng: &mut R) -> [Individual; POPULATION_COUNT]
where
    R: Rng + ?Sized,
{
    rng.random::<[Individual; POPULATION_COUNT]>()
}

fn gen_next_generation<R>(
    population: &mut [Individual; POPULATION_COUNT],
    phase: EvolutaionPhase,
    rng: &mut R,
) where
    R: Rng + ?Sized,
{
    let sigma = mutation_sigma_by_phase(phase);
    let max_w = max_weight_by_phase(phase);

    let mut next = ArrayVec::<Individual, POPULATION_COUNT>::new();

    // elite selection
    population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    next.extend(population[..ELITE_COUNT].iter().cloned());

    // generate the rest individuals
    while next.len() < POPULATION_COUNT {
        let p1 = tournament_select(population, rng);
        let p2 = tournament_select(population, rng);

        let mut child = WeightSet::blx_alpha(&p1.weights, &p2.weights, BLX_ALPHA, max_w, rng);
        child.mutate(sigma, max_w, MUTATION_RATE, rng);
        child.normalize_l1();

        next.push(Individual {
            weights: child,
            fitness: 0.0,
        });
    }

    *population = next.into_inner().unwrap();
}

fn tournament_select<'a, R>(population: &'a [Individual], rng: &mut R) -> &'a Individual
where
    R: Rng + ?Sized,
{
    // k = 2
    const _: () = assert!(TOURNAMENT_SIZE == 2);
    let a = population.choose(rng).unwrap();
    let b = population.choose(rng).unwrap();
    if a.fitness >= b.fitness { a } else { b }
}
