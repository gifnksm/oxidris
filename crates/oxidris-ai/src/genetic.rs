use std::thread;

use oxidris_stats::descriptive::DescriptiveStats;
use rand::{Rng, seq::IndexedRandom};

use oxidris_engine::GameField;

use crate::{
    board_feature::DynBoardFeatureSource, placement_evaluator::FeatureBasedPlacementEvaluator,
    session_evaluator::SessionEvaluator, turn_evaluator::TurnEvaluator, weights,
};

#[derive(Debug, Clone)]
pub struct Individual {
    weights: Vec<f32>,
    fitness: f32,
}

impl Individual {
    pub fn random<R>(rng: &mut R, max_weight: f32, feature_count: usize) -> Self
    where
        R: Rng + ?Sized,
    {
        let mut weights = weights::random(rng, max_weight, feature_count);
        weights::normalize_l1(&mut weights);
        Self {
            weights,
            fitness: f32::MIN,
        }
    }

    #[must_use]
    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    #[must_use]
    pub fn fitness(&self) -> f32 {
        self.fitness
    }
}

#[derive(Debug, Clone)]
pub struct Population {
    board_features: Vec<&'static dyn DynBoardFeatureSource>,
    individuals: Vec<Individual>,
}

impl Population {
    #[must_use]
    pub fn random<R>(
        board_features: Vec<&'static dyn DynBoardFeatureSource>,
        count: usize,
        rng: &mut R,
        max_weight: f32,
    ) -> Self
    where
        R: Rng + ?Sized,
    {
        let individuals = (0..count)
            .map(|_| Individual::random(rng, max_weight, board_features.len()))
            .collect();
        Population {
            board_features,
            individuals,
        }
    }

    #[must_use]
    pub fn individuals(&self) -> &[Individual] {
        &self.individuals
    }

    pub fn evaluate_fitness<E>(&mut self, fields: &[GameField], session_evaluator: &E)
    where
        E: SessionEvaluator + ?Sized,
    {
        thread::scope(|s| {
            for ind in &mut self.individuals {
                let placement_evaluator = FeatureBasedPlacementEvaluator::new(
                    self.board_features.clone(),
                    ind.weights.clone(),
                );
                let turn_evaluator = TurnEvaluator::new(Box::new(placement_evaluator));
                s.spawn(move || {
                    ind.fitness =
                        session_evaluator.play_and_evaluate_sessions(fields, &turn_evaluator);
                });
            }
        });

        // sort by fitness descending
        self.individuals
            .sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    }

    #[must_use]
    pub fn compute_weight_stats(&self) -> Vec<DescriptiveStats> {
        (0..self.board_features.len())
            .map(|i| {
                let weights = self.individuals().iter().map(|ind| ind.weights[i]);
                DescriptiveStats::new(weights).unwrap()
            })
            .collect()
    }

    #[must_use]
    pub fn compute_fitness_stats(&self) -> DescriptiveStats {
        DescriptiveStats::new(self.individuals.iter().map(|ind| ind.fitness)).unwrap()
    }
}

#[derive(Debug)]
pub struct PopulationEvolver {
    pub elite_count: usize,
    pub max_weight: f32,
    pub tournament_size: usize,
    pub mutation_sigma: f32,
    pub blx_alpha: f32,
    pub mutation_rate: f32,
}

impl PopulationEvolver {
    #[must_use]
    pub fn evolve(&self, population: &Population) -> Population {
        let mut rng = rand::rng();
        let mut next_individuals = vec![];
        assert!(
            population
                .individuals
                .is_sorted_by(|a, b| a.fitness >= b.fitness)
        );

        // elite selection
        next_individuals.extend(population.individuals[..self.elite_count].iter().cloned());

        // generate the rest individuals
        while next_individuals.len() < population.individuals.len() {
            let p1 = tournament_select(&population.individuals, self.tournament_size, &mut rng);
            let p2 = tournament_select(&population.individuals, self.tournament_size, &mut rng);

            let mut child = weights::blx_alpha(
                &p1.weights,
                &p2.weights,
                self.blx_alpha,
                self.max_weight,
                &mut rng,
            );
            weights::mutate(
                &mut child,
                self.mutation_sigma,
                self.max_weight,
                self.mutation_rate,
                &mut rng,
            );
            weights::normalize_l1(&mut child);

            next_individuals.push(Individual {
                weights: child,
                fitness: 0.0,
            });
        }

        Population {
            board_features: population.board_features.clone(),
            individuals: next_individuals,
        }
    }
}

fn tournament_select<'a, R>(
    population: &'a [Individual],
    tournament_size: usize,
    rng: &mut R,
) -> &'a Individual
where
    R: Rng + ?Sized,
{
    assert!(tournament_size > 0);
    population
        .choose_multiple(rng, tournament_size)
        .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
        .unwrap()
}
