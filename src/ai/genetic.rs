use std::{array, thread};

use arrayvec::ArrayVec;
use rand::{
    Rng,
    distr::{Distribution, StandardUniform},
    seq::IndexedRandom,
};

use crate::{
    ai::{
        evaluator::Evaluator,
        metrics::{HeightInfo, METRIC_COUNT},
        weights::WeightSet,
    },
    engine::state::GameState,
};

const GAMES_PER_INDIVIDUALS: usize = 3;
const MAX_PIECES_PER_GAME: usize = 800;
const LINE_SCORE: i32 = 10;
const HEIGHT_PENALTY: i32 = 5;
const HOLE_PENALTY: i32 = 8;

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
    score: i32,
}

impl Distribution<Individual> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Individual {
        let max_w = max_weight_by_phase(EvolutaionPhase::Exploration);
        Individual {
            weights: WeightSet::random(rng, max_w),
            score: i32::MIN,
        }
    }
}

impl Individual {
    fn evaluate(&mut self, games: &[GameState]) {
        let evaluator = Evaluator::new(self.weights.clone());
        self.score = 0;
        for mut game in games.iter().cloned() {
            for _ in 0..MAX_PIECES_PER_GAME {
                let Some((_, next_game)) = evaluator.select_move(&game) else {
                    break;
                };
                game = next_game;
            }
            let height_info = HeightInfo::compute(game.board());
            let cleared_lines = i32::try_from(game.total_cleared_lines()).unwrap();
            let piece_survived = i32::try_from(game.completed_pieces()).unwrap();
            let max_height = i32::from(height_info.max_height());
            let holes = i32::from(height_info.holes());
            self.score += cleared_lines * LINE_SCORE + piece_survived
                - max_height * HEIGHT_PENALTY
                - holes * HOLE_PENALTY;
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
                    println!("  {i:2}: {:.3?} => {}", ind.weights, ind.score);
                });
            }
        });

        let weights = |i| population.iter().map(move |ind| ind.weights.0[i]);
        let weights_min = WeightSet::<METRIC_COUNT>(array::from_fn(|i| min(weights(i))));
        let weights_max = WeightSet::<METRIC_COUNT>(array::from_fn(|i| max(weights(i))));
        let weights_mean = WeightSet::<METRIC_COUNT>(array::from_fn(|i| mean(weights(i))));
        let weights_stddev =
            WeightSet::<METRIC_COUNT>(array::from_fn(|i| relative_stddev(weights(i))));
        let mean_stddev = mean_weight_stddev(&population);
        let score_mean = population.iter().map(|i| i.score).sum::<i32>()
            / i32::try_from(POPULATION_COUNT).unwrap();
        let score_max = population.iter().map(|i| i.score).max().unwrap();
        let score_min = population.iter().map(|i| i.score).min().unwrap();
        println!("  Weights Min:       {:.3?}", weights_min.0);
        println!("  Weights Max:       {:.3?}", weights_max.0);
        println!("  Weights Mean:      {:.3?}", weights_mean.0);
        println!(
            "  Weights RelStddev: {:.3?} => {mean_stddev:.3}",
            weights_stddev.0
        );
        println!("  Avg Score: {score_mean}, Max Score: {score_max}, Min Score: {score_min}");

        if generation + 1 < MAX_GENERATIONS {
            gen_next_generation(&mut population, phase, &mut rng);
        }
    }

    println!("Best Individuals:");
    population.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    for (i, ind) in population.iter().take(5).enumerate() {
        println!("  {i:2}: {:?} => {}", ind.weights.0, ind.score);
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
    population.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    next.extend(population[..ELITE_COUNT].iter().cloned());

    // generate the rest individuals
    while next.len() < POPULATION_COUNT {
        let p1 = tournament_select(population, rng);
        let p2 = tournament_select(population, rng);

        let mut child = WeightSet::blx_alpha(&p1.weights, &p2.weights, BLX_ALPHA, max_w, rng);
        child.mutate(sigma, max_w, MUTATION_RATE, rng);

        next.push(Individual {
            weights: child,
            score: 0,
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
    if a.score >= b.score { a } else { b }
}
