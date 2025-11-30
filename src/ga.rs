use std::{ops::Index, process, thread};

use crossterm::event::KeyCode;
use rand::{Rng, distr::StandardUniform, prelude::Distribution, seq::SliceRandom};

use crate::{ai, game::Game, input::Input};

const POPULATION: usize = 20;
const GENERATION_MAX: usize = 20;
const LINE_COUNT_MAX: usize = 256;
const CROSSOVER_RATE: usize = 60;
const MUTATION_RATE: usize = 10;
const SELECTION_RATE: usize = 30;
const CROSSOVER_LEN: usize = POPULATION * CROSSOVER_RATE / 100;
const MUTATION_LEN: usize = POPULATION * MUTATION_RATE / 100;
const SELECTION_LEN: usize = POPULATION * SELECTION_RATE / 100;
#[allow(clippy::assertions_on_constants)]
const _: () = assert!(CROSSOVER_RATE + MUTATION_RATE + SELECTION_RATE == 100);
#[allow(clippy::assertions_on_constants)]
const _: () = assert!(CROSSOVER_LEN + MUTATION_LEN + SELECTION_LEN == POPULATION);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GenomeKind {
    Line,
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
    let _ = thread::spawn(|| {
        let mut inds = rand::random::<[Individual; POPULATION]>();
        for r#gen in 0..GENERATION_MAX {
            println!("{gen}世代目:\r");
            thread::scope(|s| {
                for (i, ind) in inds.iter_mut().enumerate() {
                    s.spawn(move || {
                        let mut game = Game::new();
                        while game.cleared_lines() < LINE_COUNT_MAX {
                            let gameover;
                            (game, gameover) = ai::eval(&game, ind.geno);
                            if gameover {
                                break;
                            }
                        }
                        ind.score = game.score();
                        println!("{i}: {:?} => {}\r", ind.geno.0, ind.score);
                    });
                }
            });
            let next_genos = gen_next_generation(&inds);
            inds.iter_mut()
                .map(|i| &mut i.geno)
                .zip(next_genos)
                .for_each(|(g, n)| *g = n);
        }
        process::exit(0);
    });

    let mut input = Input::new().unwrap();
    loop {
        if let Ok(KeyCode::Char('q')) = input.read() {
            let _ = input.cleanup();
            process::exit(0);
        }
    }
}

fn gen_next_generation(inds: &[Individual]) -> [GenoSeq; POPULATION] {
    let mut rng = rand::rng();
    let mut genos = vec![];
    genos.extend_from_slice(&crossover(inds));
    genos.extend_from_slice(&mutation(inds));
    genos.extend_from_slice(&selection(inds));
    genos.shuffle(&mut rng);
    genos.try_into().unwrap()
}

fn crossover(inds: &[Individual]) -> [GenoSeq; CROSSOVER_LEN] {
    let mut genos = inds.iter().map(|i| i.geno).collect::<Vec<_>>();
    let mut rng = rand::rng();
    for [g1, g2] in genos.as_chunks_mut::<2>().0 {
        let p1 = rng.random_range(0..4);
        let p2 = rng.random_range(p1..4);
        g1.0[p1..=p2].swap_with_slice(&mut g2.0[p1..=p2]);
    }
    genos.shuffle(&mut rng);
    genos[..CROSSOVER_LEN].try_into().unwrap()
}

fn mutation(inds: &[Individual]) -> [GenoSeq; MUTATION_LEN] {
    let mut genos = inds.iter().map(|i| i.geno).collect::<Vec<_>>();
    let mut rng = rand::rng();
    genos.shuffle(&mut rng);
    for geno in genos.iter_mut().take(MUTATION_LEN) {
        geno.0[rng.random_range(0..4)] = rand::random();
    }
    genos[..MUTATION_LEN].try_into().unwrap()
}

fn selection(inds: &[Individual]) -> [GenoSeq; SELECTION_LEN] {
    let mut new_inds = inds.to_vec();
    new_inds.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    new_inds.iter().map(|i| i.geno).collect::<Vec<_>>()[..SELECTION_LEN]
        .try_into()
        .unwrap()
}
