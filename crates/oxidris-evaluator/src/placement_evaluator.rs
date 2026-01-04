//! Placement evaluation: scoring individual piece placements.
//!
//! This module implements the first level of the evaluator architecture: assigning a score
//! to a single piece placement on the board. The score guides the AI to choose placements
//! that lead to skilled play.
//!
//! # How It Works
//!
//! Placement evaluation follows a four-step pipeline:
//!
//! 1. **Feature Extraction** - Extract features from the board state after placement
//! 2. **Feature Transformation** - Transform raw values into meaningful representations
//! 3. **Feature Normalization** - Scale transformed values to \[0.0, 1.0\] range
//! 4. **Weighted Evaluation** - Compute weighted sum: `score = Σ(wᵢ × featureᵢ)`
//!
//! # Design: Linear Weighted Sum
//!
//! The [`FeatureBasedPlacementEvaluator`] computes placement scores as a linear combination
//! of normalized features:
//!
//! ```text
//! score = w₁·f₁ + w₂·f₂ + ... + w₁₅·f₁₅
//! ```
//!
//! Where:
//!
//! - `fᵢ` is the normalized feature value ∈ \[0.0, 1.0\] (higher is always better)
//! - `wᵢ` is the learned weight (can be positive or negative)
//!
//! **Advantages:**
//!
//! - Simple and fast (just a dot product)
//! - Interpretable (can inspect individual feature contributions)
//! - Easy to train (weights learned by genetic algorithm)
//!
//! **Limitations:**
//!
//! - Cannot capture feature interactions (e.g., "holes are bad, but worse when height is high")
//! - Assumes features contribute independently to the score
//!
//! # Usage
//!
//! ```rust,no_run
//! use oxidris_evaluator::{
//!     board_feature::ALL_BOARD_FEATURES,
//!     placement_evaluator::{PlacementEvaluator, FeatureBasedPlacementEvaluator},
//! };
//!
//! // Create evaluator with features and weights
//! let features = ALL_BOARD_FEATURES.to_vec();
//! let weights = vec![1.0; features.len()]; // Example weights
//! let evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
//!
//! // Score a placement
//! // (requires PlacementAnalysis - see placement_analysis module for details)
//! // let score = evaluator.evaluate_placement(&analysis);
//! ```
//!
//! Weights are typically learned through genetic algorithm training (see `oxidris-training` crate).
//! Trained models are stored in `models/ai/` (e.g., `aggro.json`, `defensive.json`).

use std::{fmt, iter};

use crate::{board_feature::DynBoardFeatureSource, placement_analysis::PlacementAnalysis};

/// Evaluates piece placements by assigning scores.
///
/// Implementations define how to score a placement given its analysis.
/// The most common implementation is [`FeatureBasedPlacementEvaluator`], which uses
/// a weighted sum of normalized board features.
pub trait PlacementEvaluator: fmt::Debug + Send + Sync {
    /// Evaluates a placement and returns a score (higher is better).
    ///
    /// # Arguments
    /// * `analysis` - Pre-computed placement analysis containing board state and features
    ///
    /// # Returns
    /// Placement score (unbounded, typically in range \[-10.0, 10.0\])
    fn evaluate_placement(&self, analysis: &PlacementAnalysis) -> f32;
}

/// Feature-based placement evaluator using weighted sum of normalized features.
///
/// This evaluator computes placement scores as:
///
/// ```text
/// score = w₁·f₁ + w₂·f₂ + ... + w₁₅·f₁₅
/// ```
///
/// Where `fᵢ` are normalized feature values (∈ \[0.0, 1.0\]) and `wᵢ` are learned weights.
///
/// # Example
///
/// ```rust,no_run
/// use oxidris_evaluator::{
///     board_feature::ALL_BOARD_FEATURES,
///     placement_evaluator::FeatureBasedPlacementEvaluator,
/// };
///
/// let features = ALL_BOARD_FEATURES.to_vec();
/// let weights = vec![1.0; features.len()];
/// let evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
/// ```
#[derive(Debug, Clone)]
pub struct FeatureBasedPlacementEvaluator {
    features: Vec<&'static dyn DynBoardFeatureSource>,
    weights: Vec<f32>,
}

impl FeatureBasedPlacementEvaluator {
    /// Creates a new feature-based placement evaluator.
    ///
    /// # Arguments
    ///
    /// * `features` - List of features to evaluate
    /// * `weights` - Corresponding weights for each feature
    ///
    /// # Panics
    ///
    /// Panics if `features.len() != weights.len()`
    #[must_use]
    pub fn new(features: Vec<&'static dyn DynBoardFeatureSource>, weights: Vec<f32>) -> Self {
        assert_eq!(features.len(), weights.len());
        Self { features, weights }
    }
}

impl PlacementEvaluator for FeatureBasedPlacementEvaluator {
    #[inline]
    fn evaluate_placement(&self, analysis: &PlacementAnalysis) -> f32 {
        iter::zip(&self.features, &self.weights)
            .map(|(f, w)| f.compute_feature_value(analysis).normalized * w)
            .sum()
    }
}
