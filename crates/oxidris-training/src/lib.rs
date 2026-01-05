//! Training system for evolving AI feature weights using genetic algorithms.
//!
//! This crate implements the training system that optimizes feature weights for the
//! evaluator system. It uses a genetic algorithm to evolve weights that maximize
//! fitness scores defined by session evaluators.
//!
//! # How Training Works
//!
//! 1. **Population** - Create a population of individuals (each with random feature weights)
//! 2. **Evaluation** - Each individual plays multiple game sessions using its weights
//! 3. **Fitness** - Session evaluator computes fitness score for each individual
//! 4. **Selection** - Select top performers based on fitness
//! 5. **Reproduction** - Create next generation through crossover and mutation
//! 6. **Repeat** - Continue for many generations until convergence
//!
//! # Architecture
//!
//! ```text
//! Genetic Algorithm
//!     ↓ evolves
//! Feature Weights (individuals)
//!     ↓ used by
//! Placement Evaluator (oxidris-evaluator)
//!     ↓ scored by
//! Session Evaluator (fitness function)
//!     ↓ produces
//! Fitness Score
//!     ↓ guides
//! Selection & Reproduction
//! ```
//!
//! # Genetic Algorithm Parameters
//!
//! The genetic algorithm uses several key parameters:
//!
//! - **Population size** - Number of individuals per generation
//! - **Elite size** - Number of top individuals preserved unchanged
//! - **Crossover rate** - Probability of combining two parents' weights
//! - **Mutation rate** - Probability of randomly modifying weights
//! - **Mutation strength** - Magnitude of random weight changes
//!
//! See the [`genetic`] module for implementation details.
//!
//! # Fitness Functions
//!
//! Different fitness functions produce different play styles:
//!
//! - **Aggro** (`oxidris-evaluator::AggroSessionEvaluator`) - Balances survival with line clearing
//! - **Defensive** (`oxidris-evaluator::DefensiveSessionEvaluator`) - Prioritizes survival time
//!
//! # Training Process
//!
//! 1. **Generate Training Data** - Weak AI plays games to generate diverse board states (used for feature normalization percentiles)
//! 2. **Initialize Population** - Create random feature weights
//! 3. **Evolve** - Run genetic algorithm for many generations
//! 4. **Export** - Save best weights to `models/ai/*.json`
//!
//! # Example
//!
//! ```rust,ignore
//! use oxidris_training::genetic::{Population, GeneticAlgorithmParams};
//! use oxidris_evaluator::session_evaluator::{DefaultSessionEvaluator, AggroSessionEvaluator};
//! # let features = todo!(); // Build features with normalization parameters
//!
//! // Create initial population
//! let params = GeneticAlgorithmParams::default();
//! let mut population = Population::new(
//!     features,
//!     params,
//! );
//!
//! // Create fitness function
//! let fitness_fn = AggroSessionEvaluator::new();
//! let session_evaluator = DefaultSessionEvaluator::new(1000, fitness_fn);
//!
//! // Evolve for multiple generations
//! for generation in 0..100 {
//!     population.evaluate_fitness(&session_evaluator, &training_fields);
//!     population.evolve();
//! }
//!
//! // Export best individual
//! let best = population.best_individual();
//! // Save weights to models/ai/*.json
//! ```
//!
//! # Design Principles
//!
//! ## Separation from Evaluation
//!
//! Training is completely separate from evaluation:
//! - **Evaluator** defines how to score placements (what is good?)
//! - **Training** optimizes weights to maximize fitness (how to find good weights?)
//!
//! This separation allows experimenting with different training algorithms without
//! changing the evaluation system.
//!
//! ## Data-Driven Training
//!
//! Training uses actual gameplay data:
//!
//! - Training fields are generated from weak AI gameplay
//! - Feature normalization percentiles computed from real game data
//! - No hand-tuned weights or parameters
//!
//! # Current Limitations
//!
//! - **No multi-objective optimization**: Each model optimizes a single fitness function that
//!   manually combines multiple objectives (survival, score) into one scalar value. This prevents
//!   systematic exploration of trade-offs—we can't answer "what's the best survival/score balance?"
//!   or generate models with intermediate play styles without hand-designing new fitness functions.
//!   Multi-objective optimization (e.g., Pareto fronts) would reveal the full range of optimal
//!   trade-offs.
//! - **Simple GA**: Uses basic genetic algorithm without advanced techniques (e.g., adaptive
//!   mutation rates, island models, or hybrid algorithms)
//! - **No transfer learning**: Each model trained from scratch, not reusing knowledge from
//!   previously trained models
//! - **Expensive**: Requires many game simulations (parallelized but still slow)

pub mod genetic;
pub mod weights;
