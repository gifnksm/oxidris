use std::{array, thread};

use rand::{Rng, seq::IndexedRandom};

use oxidris_engine::GameField;

use crate::{
    board_feature::BoardFeatureSet, placement_evaluator::FeatureBasedPlacementEvaluator,
    session_evaluator::SessionEvaluator, statistics::Statistics, turn_evaluator::TurnEvaluator,
    weights::WeightSet,
};

#[derive(Debug, Clone)]
pub struct Individual<const FEATURE_COUNT: usize> {
    weights: WeightSet<FEATURE_COUNT>,
    fitness: f32,
}

impl<const FEATURE_COUNT: usize> Individual<FEATURE_COUNT> {
    pub fn random<R>(rng: &mut R, max_weight: f32) -> Self
    where
        R: Rng + ?Sized,
    {
        let mut weights = WeightSet::random(rng, max_weight);
        weights.normalize_l1();
        Self {
            weights,
            fitness: f32::MIN,
        }
    }

    #[must_use]
    pub fn weights(&self) -> &WeightSet<FEATURE_COUNT> {
        &self.weights
    }

    #[must_use]
    pub fn fitness(&self) -> f32 {
        self.fitness
    }
}

#[derive(Debug)]
pub struct Population<const FEATURE_COUNT: usize> {
    individuals: Vec<Individual<FEATURE_COUNT>>,
}

impl<const FEATURE_COUNT: usize> Population<FEATURE_COUNT> {
    #[must_use]
    pub fn random<R>(count: usize, rng: &mut R, max_weight: f32) -> Self
    where
        R: Rng + ?Sized,
    {
        let individuals = (0..count)
            .map(|_| Individual::random(rng, max_weight))
            .collect();
        Population { individuals }
    }

    #[must_use]
    pub fn individuals(&self) -> &[Individual<FEATURE_COUNT>] {
        &self.individuals
    }

    pub fn evaluate_fitness(
        &mut self,
        fields: &[GameField],
        session_evaluator: &dyn SessionEvaluator,
        board_features: &BoardFeatureSet<'static, FEATURE_COUNT>,
    ) {
        thread::scope(|s| {
            for ind in &mut self.individuals {
                let placement_evaluator = FeatureBasedPlacementEvaluator::new(
                    board_features.clone(),
                    ind.weights.clone(),
                );
                let turn_evaluator = TurnEvaluator::new(placement_evaluator);
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
    pub fn compute_weight_stats(&self) -> [Statistics; FEATURE_COUNT] {
        array::from_fn(|i| {
            let weights = self
                .individuals()
                .iter()
                .map(|ind| ind.weights.as_array()[i]);
            Statistics::compute(weights).unwrap()
        })
    }

    #[must_use]
    pub fn compute_fitness_stats(&self) -> Statistics {
        Statistics::compute(self.individuals.iter().map(|ind| ind.fitness)).unwrap()
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
    pub fn evolve<const FEATURE_COUNT: usize>(
        &self,
        population: &Population<FEATURE_COUNT>,
    ) -> Population<FEATURE_COUNT> {
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

            let mut child = WeightSet::blx_alpha(
                &p1.weights,
                &p2.weights,
                self.blx_alpha,
                self.max_weight,
                &mut rng,
            );
            child.mutate(
                self.mutation_sigma,
                self.max_weight,
                self.mutation_rate,
                &mut rng,
            );
            child.normalize_l1();

            next_individuals.push(Individual {
                weights: child,
                fitness: 0.0,
            });
        }

        Population {
            individuals: next_individuals,
        }
    }
}

fn tournament_select<'a, R, const FEATURE_COUNT: usize>(
    population: &'a [Individual<FEATURE_COUNT>],
    tournament_size: usize,
    rng: &mut R,
) -> &'a Individual<FEATURE_COUNT>
where
    R: Rng + ?Sized,
{
    assert!(tournament_size > 0);
    population
        .choose_multiple(rng, tournament_size)
        .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
        .unwrap()
}
