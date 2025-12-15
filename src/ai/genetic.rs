use std::{array, iter, thread};

use arrayvec::ArrayVec;
use rand::{
    Rng,
    distr::{Distribution, StandardUniform},
    seq::IndexedRandom,
};

use crate::{
    ai::{evaluator::Evaluator, metrics::METRIC_COUNT, weights::WeightSet},
    engine::state::GameState,
};

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

impl Distribution<Individual> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Individual {
        let max_w = max_weight_by_phase(EvolutaionPhase::Exploration);
        Individual {
            weights: WeightSet::random(rng, max_w),
            fitness: f32::MIN,
        }
    }
}

const LINE_CLEAR_WEIGHT: [u16; 5] = [0, 1, 3, 5, 8];

impl Individual {
    fn evaluate(&mut self, games: &[GameState]) {
        let evaluator = Evaluator::new(self.weights.clone());
        self.fitness = 0.0;
        for mut game in games.iter().cloned() {
            for _ in 0..MAX_PIECES_PER_GAME {
                let Some((_, next_game)) = evaluator.select_move(&game) else {
                    break;
                };
                game = next_game;
            }

            #[expect(clippy::cast_precision_loss)]
            let survived = game.completed_pieces() as f32;
            #[expect(clippy::cast_precision_loss)]
            let max_pieces = MAX_PIECES_PER_GAME as f32;
            let survived_ratio = survived / max_pieces;
            let survival_bonus = 2.0 * survived_ratio * survived_ratio;
            #[expect(clippy::cast_precision_loss)]
            let weighted_line_count = iter::zip(LINE_CLEAR_WEIGHT, game.line_cleared_counter())
                .map(|(w, c)| f32::from(w) * (*c as f32))
                .sum::<f32>();
            let efficiency = weighted_line_count / survived.max(1.0);
            self.fitness += survival_bonus + efficiency * survived_ratio;
        }
    }
}

pub(crate) fn learning() {
    let mut rng = rand::rng();
    let mut population = rng.random::<[Individual; POPULATION_COUNT]>();
    for generation in 0..MAX_GENERATIONS {
        let phase = EvolutaionPhase::from_generation(generation);
        println!("Generation #{generation} ({phase:?}):");
        let games: &[_; GAMES_PER_INDIVIDUALS] = &array::from_fn(|_| GameState::new());
        thread::scope(|s| {
            for (i, ind) in population.iter_mut().enumerate() {
                s.spawn(move || {
                    ind.evaluate(games);
                    println!("  {i:2}: {:.3?} => {}", ind.weights, ind.fitness);
                });
            }
        });

        #[expect(clippy::cast_precision_loss)]
        let population_count = POPULATION_COUNT as f32;

        let weights = |i| population.iter().map(move |ind| ind.weights.0[i]);
        let weights_min = WeightSet::<METRIC_COUNT>(array::from_fn(|i| min(weights(i))));
        let weights_max = WeightSet::<METRIC_COUNT>(array::from_fn(|i| max(weights(i))));
        let weights_mean = WeightSet::<METRIC_COUNT>(array::from_fn(|i| mean(weights(i))));
        let weights_stddev =
            WeightSet::<METRIC_COUNT>(array::from_fn(|i| relative_stddev(weights(i))));
        let mean_stddev = mean_weight_stddev(&population);

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
        println!("    Min:       {:.3?}", weights_min.0);
        println!("    Max:       {:.3?}", weights_max.0);
        println!("    Mean:      {:.3?}", weights_mean.0);
        println!(
            "    RelStddev: {:.3?} => {mean_stddev:.3}",
            weights_stddev.0
        );
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
        println!("  {i:2}: {:?} => {}", ind.weights.0, ind.fitness);
    }
}

fn mean_weight_stddev(inds: &[Individual]) -> f32 {
    let weights: [f32; METRIC_COUNT] =
        array::from_fn(|i| relative_stddev(inds.iter().map(|ind| ind.weights.0[i])));
    mean(weights)
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

fn mean_stddev(values: impl IntoIterator<Item = f32> + Clone) -> (f32, f32) {
    let m = mean(values.clone());
    let variance = mean(values.into_iter().map(|x| (x - m).powi(2)));
    (m, variance.sqrt())
}

fn relative_stddev(values: impl IntoIterator<Item = f32> + Clone) -> f32 {
    let (m, s) = mean_stddev(values);
    s / m
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
