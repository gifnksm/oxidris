//! Genetic algorithm implementation for evolving feature weights.
//!
//! This module implements a genetic algorithm (GA) that evolves populations of feature
//! weights to maximize fitness scores. The GA uses tournament selection, BLX-α crossover,
//! and Gaussian mutation to create new generations.
//!
//! # Algorithm Overview
//!
//! The genetic algorithm follows this cycle:
//!
//! 1. **Evaluate Fitness** - Each individual plays game sessions and receives a fitness score
//! 2. **Elite Selection** - Top performers are preserved unchanged in the next generation
//! 3. **Tournament Selection** - Select parents for reproduction using tournament selection
//! 4. **Crossover (BLX-α)** - Combine two parents' weights to create offspring
//! 5. **Mutation** - Apply random Gaussian noise to weights
//! 6. **Normalization** - Normalize weights to sum to 1.0 (L1 normalization)
//!
//! # Key Components
//!
//! - [`Individual`] - A single candidate solution (set of feature weights + fitness score)
//! - [`Population`] - Collection of individuals evaluated together
//! - [`PopulationEvolver`] - Controls evolution parameters (selection, crossover, mutation)
//!
//! # Genetic Operators
//!
//! ## Tournament Selection
//!
//! Randomly select K individuals and choose the one with highest fitness. This creates
//! selection pressure while maintaining diversity (larger K = stronger pressure).
//!
//! ## BLX-α Crossover
//!
//! Blend crossover that interpolates between two parent weights with exploration beyond
//! the parent range (α controls exploration distance). Produces offspring that can explore
//! regions near both parents.
//!
//! ## Gaussian Mutation
//!
//! Adds random noise from N(0, σ²) to each weight with probability `mutation_rate`.
//! This introduces new variations and prevents premature convergence.
//!
//! # Parallelization
//!
//! Fitness evaluation is parallelized using threads - each individual evaluates its
//! fitness independently across multiple game sessions.
//!
//! # Example
//!
//! ```rust,ignore
//! use oxidris_training::genetic::{Population, PopulationEvolver};
//!
//! // Create initial population
//! let mut population = Population::random(features, 30, &mut rng, 10.0);
//!
//! // Create evolver with parameters
//! let evolver = PopulationEvolver {
//!     elite_count: 3,
//!     max_weight: 10.0,
//!     tournament_size: 2,
//!     mutation_sigma: 0.5,
//!     blx_alpha: 0.5,
//!     mutation_rate: 0.1,
//! };
//!
//! // Evolution loop
//! for generation in 0..100 {
//!     population.evaluate_fitness(&fields, &session_evaluator);
//!     population = evolver.evolve(&population);
//! }
//! ```
//!
//! # Design Decisions
//!
//! ## L1 Weight Normalization
//!
//! All weights are normalized to sum to 1.0 (L1 normalization) after initialization and
//! after each genetic operation. This serves multiple purposes:
//!
//! - **Scale invariance**: Eliminates redundant solutions where weights differ only by a
//!   constant multiplier (e.g., `[1, 2, 3]` and `[2, 4, 6]` produce identical placement
//!   rankings). This reduces the search space and helps the GA converge faster.
//! - **Comparability**: Ensures fitness scores across individuals reflect strategy differences
//!   rather than weight magnitude differences.
//! - **Bounded growth**: Prevents any single feature weight from growing unbounded, which
//!   could make other features irrelevant.
//!
//! ## Parameter Control
//!
//! Evolution parameters (mutation rate, tournament size, etc.) are provided via
//! `PopulationEvolver` for each generation. The algorithm itself does not automatically
//! adapt parameters, but callers can implement adaptive strategies by creating different
//! evolver instances per generation (e.g., reducing mutation rate over time).
//!
//! # Current Limitations
//!
//! - **No automatic parameter adaptation**: The algorithm does not automatically adjust
//!   mutation rates or selection pressure based on population state. Adaptive strategies
//!   must be implemented externally by the caller (creating different evolver instances
//!   per generation)
//! - **Limited diversity maintenance**: Only elitism and tournament selection maintain
//!   diversity; no explicit diversity preservation mechanisms (e.g., fitness sharing,
//!   crowding)
//! - **No restart mechanism**: If the population converges to a local optimum, there's
//!   no automatic restart or perturbation strategy
//! - **Single-objective only**: Implementation assumes scalar fitness values; cannot
//!   handle multi-objective optimization (e.g., Pareto fronts)
//! - **No parameter guidance**: Choosing appropriate values for tournament size, mutation
//!   rates, etc. requires manual experimentation
//!
//! See the crate-level documentation for broader training system limitations.

use std::thread;

use oxidris_stats::descriptive::DescriptiveStats;
use rand::{Rng, seq::IndexedRandom};

use oxidris_engine::GameField;
use oxidris_evaluator::{
    board_feature::BoxedBoardFeature, placement_evaluator::FeatureBasedPlacementEvaluator,
    session_evaluator::SessionEvaluator, turn_evaluator::TurnEvaluator,
};

use crate::weights;

/// A single individual in the genetic algorithm population.
///
/// An individual represents a candidate solution: a set of feature weights and its
/// associated fitness score. Fitness is computed by playing game sessions using
/// these weights.
#[derive(Debug, Clone)]
pub struct Individual {
    weights: Vec<f32>,
    fitness: f32,
}

impl Individual {
    /// Creates a new individual with random weights.
    ///
    /// Weights are uniformly distributed in [0, `max_weight`] and then L1-normalized
    /// (sum to 1.0).
    ///
    /// # Arguments
    ///
    /// * `rng` - Random number generator
    /// * `max_weight` - Maximum value for any single weight before normalization
    /// * `feature_count` - Number of features (length of weight vector)
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

    /// Returns the feature weights for this individual.
    #[must_use]
    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    /// Returns the fitness score for this individual.
    ///
    /// Fitness is computed by playing game sessions and evaluating performance.
    /// Higher fitness means better performance.
    #[must_use]
    pub fn fitness(&self) -> f32 {
        self.fitness
    }
}

/// A population of individuals for genetic algorithm evolution.
///
/// The population manages a collection of individuals (candidate solutions) and
/// provides methods for fitness evaluation and statistics computation.
#[derive(Debug, Clone)]
pub struct Population {
    board_features: Vec<BoxedBoardFeature>,
    individuals: Vec<Individual>,
}

impl Population {
    /// Creates a new population with random individuals.
    ///
    /// # Arguments
    ///
    /// * `board_features` - Features to evaluate (defines weight vector length)
    /// * `count` - Number of individuals in the population
    /// * `rng` - Random number generator
    /// * `max_weight` - Maximum weight value before normalization
    #[must_use]
    pub fn random<R>(
        board_features: Vec<BoxedBoardFeature>,
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

    /// Returns all individuals in this population.
    #[must_use]
    pub fn individuals(&self) -> &[Individual] {
        &self.individuals
    }

    /// Evaluates fitness for all individuals in parallel.
    ///
    /// Each individual plays game sessions using its weights and receives a fitness
    /// score from the session evaluator. After evaluation, individuals are sorted
    /// by fitness in descending order (best first).
    ///
    /// # Arguments
    ///
    /// * `fields` - Training game fields to evaluate on
    /// * `session_evaluator` - Fitness function to evaluate game sessions
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

    /// Computes descriptive statistics for each weight across all individuals.
    ///
    /// Returns statistics (mean, std, min, max, etc.) for each feature weight,
    /// useful for analyzing population diversity and convergence.
    #[must_use]
    pub fn compute_weight_stats(&self) -> Vec<DescriptiveStats> {
        (0..self.board_features.len())
            .map(|i| {
                let weights = self.individuals().iter().map(|ind| ind.weights[i]);
                DescriptiveStats::new(weights).unwrap()
            })
            .collect()
    }

    /// Computes descriptive statistics for fitness across all individuals.
    ///
    /// Returns statistics for the entire population's fitness distribution,
    /// useful for tracking training progress.
    #[must_use]
    pub fn compute_fitness_stats(&self) -> DescriptiveStats {
        DescriptiveStats::new(self.individuals.iter().map(|ind| ind.fitness)).unwrap()
    }
}

/// Controls genetic algorithm evolution parameters.
///
/// This struct defines how populations evolve from one generation to the next,
/// including selection pressure, crossover behavior, and mutation rates.
#[derive(Debug)]
pub struct PopulationEvolver {
    /// Number of top individuals preserved unchanged (elitism)
    pub elite_count: usize,
    /// Maximum allowed weight value (weights are clipped to [0, `max_weight`])
    pub max_weight: f32,
    /// Tournament size for selection (larger = stronger selection pressure)
    pub tournament_size: usize,
    /// Standard deviation for Gaussian mutation noise
    pub mutation_sigma: f32,
    /// BLX-α crossover parameter (controls exploration beyond parent range)
    pub blx_alpha: f32,
    /// Probability of mutating each weight (per-weight mutation rate)
    pub mutation_rate: f32,
}

impl PopulationEvolver {
    /// Evolves the population to create the next generation.
    ///
    /// 1. Preserves top `elite_count` individuals unchanged
    /// 2. Creates remaining individuals through tournament selection, crossover, and mutation
    /// 3. All new weights are L1-normalized
    ///
    /// # Arguments
    ///
    /// * `population` - Current population (must be sorted by fitness descending)
    ///
    /// # Returns
    ///
    /// New population with same size as input
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

/// Selects an individual using tournament selection.
///
/// Randomly selects `tournament_size` individuals and returns the one with
/// the highest fitness. Larger tournament sizes create stronger selection
/// pressure toward high-fitness individuals.
///
/// # Arguments
///
/// * `population` - Pool of individuals to select from
/// * `tournament_size` - Number of individuals in each tournament
/// * `rng` - Random number generator
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
