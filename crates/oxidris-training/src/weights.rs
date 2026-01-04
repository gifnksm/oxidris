//! Weight vector operations for genetic algorithm.
//!
//! This module provides utility functions for creating, manipulating, and normalizing
//! weight vectors used in the genetic algorithm training process. These operations are
//! used by [`genetic::PopulationEvolver`](crate::genetic::PopulationEvolver) to implement
//! initialization, crossover, mutation, and normalization steps.
//!
//! # Operations
//!
//! - **Initialization**: [`random`] generates random weight vectors
//! - **Crossover**: [`blx_alpha`] implements BLX-α crossover operator
//! - **Mutation**: [`mutate`] applies Gaussian mutation
//! - **Normalization**: [`normalize_l1`] performs L1 normalization
//!
//! # Design Decisions
//!
//! ## BLX-α Crossover
//!
//! We use BLX-α (Blend Crossover with alpha parameter) for the following reasons:
//!
//! - **Exploration capability**: Unlike uniform crossover, BLX-α can generate offspring
//!   outside the parent range, enabling discovery of better solutions not present in parents
//! - **Tunable exploration**: The `alpha` parameter controls range expansion (typical: 0.5)
//! - **Real-valued optimization**: Designed specifically for continuous parameter spaces
//!
//! ## Gaussian Mutation
//!
//! Gaussian (normal distribution) mutation provides:
//!
//! - **Gradual refinement**: Small changes are more likely than large jumps
//! - **Tunable strength**: `sigma` controls mutation magnitude
//! - **Probabilistic application**: `rate` controls how many weights are mutated
//!
//! ## L1 Normalization
//!
//! L1 normalization (weights sum to 1.0) provides several benefits:
//!
//! - **Scale invariance**: Eliminates redundant solutions differing only by a constant factor
//! - **Bounded search space**: Constrains the optimization space to a simplex
//! - **Comparability**: Weights represent relative importance, making them interpretable
//!
//! See [`oxidris_evaluator::session_evaluator`] for how normalized weights are used in
//! fitness evaluation.
//!
//! # Related
//!
//! - [`genetic`](crate::genetic) module uses these operations to evolve populations
//! - [`oxidris_evaluator::session_evaluator`] explains weight interpretation and fitness

use rand::Rng;
use rand_distr::Normal;

/// Creates a weight vector by applying a function to each index.
///
/// This is a generic builder function for constructing weight vectors with custom
/// initialization logic. It's useful for implementing custom initialization strategies
/// beyond simple random initialization.
///
/// # Arguments
///
/// * `f` - Function mapping index to weight value
/// * `len` - Number of weights to generate
///
/// # Examples
///
/// ```
/// use oxidris_training::weights;
///
/// // Initialize with decreasing importance
/// let weights = weights::from_fn(|i| 1.0 / (i as f32 + 1.0), 5);
/// assert_eq!(weights.len(), 5);
/// assert_eq!(weights[0], 1.0);
/// assert_eq!(weights[1], 0.5);
///
/// // Initialize with constant value
/// let weights = weights::from_fn(|_| 0.5, 3);
/// assert_eq!(weights, vec![0.5, 0.5, 0.5]);
/// ```
pub fn from_fn<F>(mut f: F, len: usize) -> Vec<f32>
where
    F: FnMut(usize) -> f32,
{
    let mut values = Vec::with_capacity(len);
    for i in 0..len {
        values.push(f(i));
    }
    values
}

/// Generates a random weight vector with uniform distribution.
///
/// Each weight is independently sampled from the range `[0.0, max_weight]`.
/// This is used for initial population generation in the genetic algorithm.
///
/// # Arguments
///
/// * `rng` - Random number generator
/// * `max_weight` - Maximum value for any weight
/// * `len` - Number of weights to generate
///
/// # Returns
///
/// A vector of `len` random weights in `[0.0, max_weight]`
pub fn random<R>(rng: &mut R, max_weight: f32, len: usize) -> Vec<f32>
where
    R: Rng + ?Sized,
{
    from_fn(|_| rng.random_range(0.0..=max_weight), len)
}

/// Performs BLX-α (Blend Crossover) between two parent weight vectors.
///
/// BLX-α generates offspring by sampling uniformly from an expanded range around
/// each parent pair. For parents `x1` and `x2` at position `i`:
///
/// 1. Compute `d = |x2 - x1|` (distance between parents)
/// 2. Expand range: `[min - α·d, max + α·d]`
/// 3. Sample offspring uniformly from expanded range
/// 4. Clamp to `[0.0, max_weight]`
///
/// The `alpha` parameter controls exploration beyond the parent range:
///
/// - `alpha = 0.0`: Offspring strictly between parents (no exploration)
/// - `alpha = 0.5`: Standard BLX-0.5 (recommended, moderate exploration)
/// - `alpha > 0.5`: Aggressive exploration (may destabilize convergence)
///
/// # Arguments
///
/// * `p1` - First parent weight vector
/// * `p2` - Second parent weight vector
/// * `alpha` - Range expansion factor (typical: 0.5)
/// * `max_weight` - Maximum value for any weight
/// * `rng` - Random number generator
///
/// # Panics
///
/// Panics if parent vectors have different lengths.
///
/// # Returns
///
/// A new weight vector combining genes from both parents with exploration
pub fn blx_alpha<R>(p1: &[f32], p2: &[f32], alpha: f32, max_weight: f32, rng: &mut R) -> Vec<f32>
where
    R: Rng + ?Sized,
{
    assert_eq!(p1.len(), p2.len());
    from_fn(
        |i| {
            let x1 = p1[i];
            let x2 = p2[i];
            let min = f32::min(x1, x2);
            let max = f32::max(x1, x2);
            let d = max - min;
            let lower = min - alpha * d;
            let upper = max + alpha * d;
            rng.random_range(lower..=upper).clamp(0.0, max_weight)
        },
        p1.len(),
    )
}

/// Applies Gaussian mutation to a weight vector in-place.
///
/// For each weight, with probability `rate`:
///
/// 1. Sample a perturbation from `N(0, sigma)`
/// 2. Add perturbation to the weight
/// 3. Clamp result to `[0.0, max_weight]`
///
/// Gaussian mutation provides gradual refinement with small changes more likely than
/// large jumps. The standard deviation `sigma` controls mutation strength.
///
/// # Arguments
///
/// * `weights` - Weight vector to mutate (modified in-place)
/// * `sigma` - Standard deviation of Gaussian perturbation
/// * `max_weight` - Maximum value for any weight
/// * `rate` - Probability of mutating each weight (typical: 0.1–0.3)
/// * `rng` - Random number generator
///
/// # Example Parameter Values
///
/// Typical configurations from `train_ai.rs`:
///
/// - **Exploration phase**: `sigma = 1.0`, `rate = 0.3` (high variation)
/// - **Transition phase**: `sigma = 0.5`, `rate = 0.2` (moderate refinement)
/// - **Convergence phase**: `sigma = 0.1`, `rate = 0.1` (fine-tuning)
pub fn mutate<R>(weights: &mut [f32], sigma: f32, max_weight: f32, rate: f32, rng: &mut R)
where
    R: Rng + ?Sized,
{
    let normal = Normal::new(0.0, sigma).unwrap();
    for w in weights {
        if rng.random_bool(rate.into()) {
            *w = (*w + rng.sample(normal)).clamp(0.0, max_weight);
        }
    }
}

/// Normalizes a weight vector to sum to 1.0 (L1 normalization).
///
/// This enforces the constraint that weights represent relative importance rather than
/// absolute magnitudes. After normalization:
///
/// - Each weight represents its proportional contribution
/// - The search space is constrained to a simplex
/// - Solutions differing only by a constant multiplier become identical
///
/// If the sum is zero or negative, weights are left unchanged (to avoid division by zero).
///
/// # Design Rationale
///
/// L1 normalization provides:
///
/// - **Scale invariance**: Eliminates redundant solutions like `[1, 2, 3]` vs `[2, 4, 6]`
/// - **Bounded growth**: Prevents unbounded weight magnitudes during evolution
/// - **Interpretability**: Normalized weights clearly show relative importance
///
/// See [`oxidris_evaluator::session_evaluator`] for how fitness functions use normalized
/// weights to compare solutions.
///
/// # Arguments
///
/// * `weights` - Weight vector to normalize (modified in-place)
pub fn normalize_l1(weights: &mut [f32]) {
    let sum: f32 = weights.iter().copied().sum();
    if sum > 0.0 {
        for w in weights {
            *w /= sum;
        }
    }
}
