use std::{array, iter, thread};

use rand::{Rng, distr::StandardUniform, prelude::Distribution, seq::SliceRandom};

use crate::{
    ai::{evaluator::Evaluator, metrics::METRIC_COUNT, weights::WeightSet},
    engine::state::GameState,
};

const GAME_COUNT: usize = 5;
const POPULATION: usize = 30;
const GENERATION_MAX: usize = 30;
const PIECE_COUNT: usize = 500;
const MUTATION_RATE: f64 = 0.02;
const SELECTION_RATE: u32 = 20;
const SELECTION_LEN: usize = POPULATION * (SELECTION_RATE as usize) / 100;

#[derive(Debug, Clone)]
struct Individual {
    weights: WeightSet<METRIC_COUNT>,
    score: usize,
}

impl Distribution<Individual> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Individual {
        Individual {
            weights: rng.random(),
            score: 0,
        }
    }
}

pub(crate) fn learning() {
    let mut inds = rand::random::<[Individual; POPULATION]>();
    for r#gen in 0..GENERATION_MAX {
        println!("{gen}th Generation:");
        let games = iter::repeat_with(GameState::new)
            .take(GAME_COUNT)
            .collect::<Vec<_>>();
        thread::scope(|s| {
            for (i, ind) in inds.iter_mut().enumerate() {
                let evaluator = Evaluator::new(ind.weights.clone());
                let games = &games;
                s.spawn(move || {
                    ind.score = 0;
                    for mut game in games.iter().cloned() {
                        for _ in 0..PIECE_COUNT {
                            let Some((_, next_game)) = evaluator.select_move(&game) else {
                                break;
                            };
                            game = next_game;
                        }
                        ind.score += game.score();
                    }
                    ind.score /= GAME_COUNT;
                    println!("  {i:2}: {:?} => {}", ind.weights, ind.score);
                });
            }
        });

        let avg_score: usize = inds.iter().map(|i| i.score).sum::<usize>() / POPULATION;
        let max_score = inds.iter().map(|i| i.score).max().unwrap();
        let min_score = inds.iter().map(|i| i.score).min().unwrap();
        let mean_stddev = mean_weight_stddev(&inds);
        println!("  Mean Weight Stddev: {mean_stddev}");
        println!("  Avg Score: {avg_score}, Max Score: {max_score}, Min Score: {min_score}");

        gen_next_generation(&mut inds);
    }

    println!("Best Individuals:");
    inds.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    for (i, ind) in inds.iter().take(5).enumerate() {
        println!("  {i:2}: {:?} => {}", ind.weights.0, ind.score);
    }
}

fn mean_weight_stddev(inds: &[Individual]) -> f32 {
    let weights: [f32; METRIC_COUNT] =
        array::from_fn(|i| relative_stddev(inds.iter().map(|ind| ind.weights.0[i].0)));
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

fn mean_stddev(values: impl IntoIterator<Item = f32> + Clone) -> (f32, f32) {
    let m = mean(values.clone());
    let variance = mean(values.into_iter().map(|x| (x - m).powi(2)));
    (m, variance.sqrt())
}

fn relative_stddev(values: impl IntoIterator<Item = f32> + Clone) -> f32 {
    let (m, s) = mean_stddev(values);
    s / m
}

fn gen_next_generation(inds: &mut [Individual; POPULATION]) {
    inds.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let mut next_inds = vec![];

    // drop the worst individuals
    let selected_inds = &mut inds[..POPULATION - SELECTION_LEN];

    // keep the best individuals
    next_inds.extend_from_slice(&selected_inds[..SELECTION_LEN]);

    // crossover and mutation for the reset individuals
    crossover(selected_inds);
    mutation(selected_inds);
    next_inds.extend_from_slice(selected_inds);

    inds.clone_from_slice(&next_inds);
}

fn crossover(inds: &mut [Individual]) {
    let mut rng = rand::rng();
    inds.shuffle(&mut rng);
    for [i1, i2] in inds.as_chunks_mut().0 {
        [i1.weights, i2.weights] =
            WeightSet::two_point_crossover(&i1.weights, &i2.weights, &mut rng);
    }
}

fn mutation(inds: &mut [Individual]) {
    let mut rng = rand::rng();
    for ind in inds.iter_mut() {
        ind.weights = ind.weights.mutation(&mut rng, MUTATION_RATE);
    }
}
