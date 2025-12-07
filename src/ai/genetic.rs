use std::{iter, mem, ops::Index, thread};

use rand::{
    Rng,
    distr::StandardUniform,
    prelude::Distribution,
    seq::{IndexedMutRandom, SliceRandom},
};

use crate::{ai::evaluator, engine::state::GameState};

const GAME_COUNT: usize = 5;
const POPULATION: usize = 30;
const GENERATION_MAX: usize = 30;
const PIECE_COUNT: usize = 500;
const MUTATION_RATE: u32 = 20;
const SELECTION_RATE: u32 = 20;
const SELECTION_LEN: usize = POPULATION * (SELECTION_RATE as usize) / 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenomeKind {
    LinesCleared,
    HeightMax,
    HeightDiff,
    DeadSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GenoSeq(pub [u16; 4]);

impl Index<GenomeKind> for GenoSeq {
    type Output = u16;

    fn index(&self, kind: GenomeKind) -> &Self::Output {
        &self.0[kind as usize]
    }
}

impl Distribution<GenoSeq> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> GenoSeq {
        GenoSeq(rng.random())
    }
}

#[derive(Debug, Clone, Copy)]
struct Individual {
    geno: GenoSeq,
    score: usize,
}

impl Distribution<Individual> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Individual {
        Individual {
            geno: rng.random(),
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
                let games = &games;
                s.spawn(move || {
                    ind.score = 0;
                    for mut game in games.iter().cloned() {
                        for _ in 0..PIECE_COUNT {
                            let Some((_, next_game)) = evaluator::eval(&game, ind.geno) else {
                                break;
                            };
                            game = next_game;
                        }
                        ind.score += game.score();
                    }
                    ind.score /= GAME_COUNT;
                    println!("  {i:2}: {:?} => {}", ind.geno.0, ind.score);
                });
            }
        });

        let avg_score: usize = inds.iter().map(|i| i.score).sum::<usize>() / POPULATION;
        let max_score = inds.iter().map(|i| i.score).max().unwrap();
        let min_score = inds.iter().map(|i| i.score).min().unwrap();
        println!("  Avg Score: {avg_score}, Max Score: {max_score}, Min Score: {min_score}");

        gen_next_generation(&mut inds);
    }
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

    inds.copy_from_slice(&next_inds);
}

fn crossover(inds: &mut [Individual]) {
    let mut rng = rand::rng();
    inds.shuffle(&mut rng);
    for [i1, i2] in inds.as_chunks_mut().0 {
        let mut idx = [0, 1, 2, 3];
        idx.shuffle(&mut rng);
        mem::swap(&mut i1.geno.0[idx[0]], &mut i2.geno.0[idx[0]]);
        mem::swap(&mut i1.geno.0[idx[1]], &mut i2.geno.0[idx[1]]);
    }
}

fn mutation(inds: &mut [Individual]) {
    let mut rng = rand::rng();
    for ind in inds.iter_mut() {
        if rand::random_ratio(MUTATION_RATE, 1000) {
            *ind.geno.0.choose_mut(&mut rng).unwrap() = rand::random();
        }
    }
}
