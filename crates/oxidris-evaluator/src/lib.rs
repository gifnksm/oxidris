//! Evaluator system for scoring Tetris piece placements and game sessions.
//!
//! This crate implements a three-level evaluation architecture:
//!
//! 1. **Placement Evaluation** ([`placement_evaluator`]) - Scores individual piece placements
//!    using feature extraction, transformation, normalization, and weighted evaluation.
//!
//! 2. **Turn Evaluation** ([`turn_evaluator`]) - Selects the best placement for the current
//!    turn by evaluating all valid placements and choosing the highest-scoring option.
//!
//! 3. **Session Evaluation** ([`session_evaluator`]) - Evaluates entire game sessions using
//!    fitness functions (Aggro/Defensive) for training the genetic algorithm.
//!
//! # Architecture
//!
//! ```text
//! Session Evaluation (fitness for training)
//!     ↓ uses
//! Turn Evaluation (select best placement)
//!     ↓ uses
//! Placement Evaluation (score single placement)
//! ```
//!
//! The placement evaluator extracts and processes board features ([`board_feature`], 15 features
//! covering survival, structure, and score aspects) to produce a score for each placement.
//!
//! # Supporting Modules
//!
//! - [`board_analysis`] - Lazy-evaluated board metrics (heights, holes, transitions, etc.)
//!   used by features to extract raw values efficiently
//! - [`placement_analysis`] - Analyzes board state after piece placement (combines line clears
//!   with board analysis)
//!
//! # Design Principles
//!
//! ## Data-Driven Evaluation
//!
//! Features use percentile-based normalization (P05-P95) computed from actual gameplay data.
//! This makes the evaluation robust to outliers and grounded in real game behavior.
//!
//! ## Separation of Concerns
//!
//! - **Features** measure board properties (what is the state?)
//! - **Placement Evaluator** scores placements (how good is this placement?)
//! - **Turn Evaluator** selects moves (which placement should I choose?)
//! - **Session Evaluator** defines objectives (what makes a good game?)
//!
//! ## Linear Evaluation Model
//!
//! Placement scores are computed as weighted sums of normalized features. This is simple,
//! interpretable, and fast, but cannot capture feature interactions (non-linear relationships).
//!
//! # Example: Using the Evaluator
//!
//! ```rust,no_run
//! use oxidris_evaluator::{
//!     placement_evaluator::{PlacementEvaluator, FeatureBasedPlacementEvaluator},
//!     turn_evaluator::TurnEvaluator,
//! };
//! # let features = todo!(); // Build features with normalization parameters
//! # let weights = todo!(); // Load trained weights
//!
//! // Create placement evaluator with features and weights
//! let placement_evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
//!
//! // Create turn evaluator
//! let turn_evaluator = TurnEvaluator::new(Box::new(placement_evaluator));
//!
//! // Select best turn for current game state
//! // (requires game field state - see turn_evaluator module docs for complete example)
//! // let best_turn = turn_evaluator.select_best_turn(&field);
//! ```
//!
//! # Current Limitations
//!
//! - **Linear transformation**: Most features use linear transformation (`raw as f32`), which
//!   doesn't capture non-linear relationships between feature values and survival time.
//! - **Feature redundancy**: Some survival features are duplicated with different normalization
//!   ranges to approximate non-linearity (ad-hoc solution).
//! - **No feature interactions**: Linear weighted sum cannot capture interactions between features.
//!
//! See the project documentation for ongoing improvements and design discussions.

pub mod board_analysis;
pub mod board_feature;
pub mod placement_analysis;
pub mod placement_evaluator;
pub mod session_evaluator;
pub mod turn_evaluator;
