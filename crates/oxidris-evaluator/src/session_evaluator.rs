//! Session evaluation: fitness functions for training AI models.
//!
//! This module implements the third level of the evaluator architecture: evaluating entire
//! game sessions to compute fitness scores for genetic algorithm training. Different fitness
//! functions produce models with different play styles.
//!
//! # How It Works
//!
//! Session evaluation involves:
//!
//! 1. **Play Session** - AI plays a complete game (up to turn limit)
//! 2. **Collect Statistics** - Track survival time, lines cleared, max height, etc.
//! 3. **Compute Fitness** - Apply fitness function to statistics
//!
//! The fitness score represents how "good" the AI played according to the fitness function's
//! objectives. During training, the genetic algorithm evolves feature weights to maximize
//! this fitness score.
//!
//! # Fitness Functions
//!
//! ## Aggro Session Evaluator
//!
//! Balances survival time with line clearing efficiency:
//!
//! ```text
//! fitness = survival_bonus + efficiency × survival_ratio - height_penalty
//!
//! where:
//!   survival_bonus = 2.0 × (survived / turn_limit)²
//!   efficiency = weighted_lines / survived_pieces
//!   height_penalty = max(worst_height - 10, 0) / 5.0
//! ```
//!
//! **Characteristics:**
//!
//! - Quadratic survival bonus (surviving longer is exponentially better)
//! - Rewards efficient line clearing with exponential weights (4-line Tetris gets 8× value)
//! - Penalizes risky high stacks above height 10
//! - Efficiency scaled by survival ratio (rewards sustained performance)
//!
//! ## Defensive Session Evaluator
//!
//! Prioritizes survival time above all else:
//!
//! ```text
//! fitness = survival_bonus + efficiency × survival_ratio - height_penalty
//!
//! where:
//!   survival_bonus = 2.0 × (survived / turn_limit)²
//!   efficiency = total_lines / survived_pieces (unweighted, unlike Aggro)
//!   height_penalty = worst_height / 20.0
//! ```
//!
//! **Characteristics:**
//!
//! - Same quadratic survival bonus (primary objective)
//! - Efficiency uses unweighted line count (all clears valued equally, unlike Aggro)
//! - Smaller height penalty (more tolerant of height)
//! - Focus: maximize survival time, clear lines as secondary goal
//!
//! # Design: Defining "Good Play"
//!
//! Different session evaluators define different objectives:
//!
//! - **Aggro**: "Good play means surviving while clearing lines efficiently"
//! - **Defensive**: "Good play means surviving as long as possible"
//!
//! These objectives drive the genetic algorithm to learn different feature weights,
//! resulting in models with different play styles despite using the same features.
//!
//! ## Design Rationale and Limitations
//!
//! **Current Approach:** The fitness formulas and coefficients (e.g., `2.0 × survival_ratio²`,
//! line clear weights `[0,1,3,5,8]`, height penalty thresholds) were chosen manually based on
//! intuition about what "good play" means. The quadratic survival bonus emphasizes consistency,
//! and the line clear weights encourage multi-line clears (especially 4-line Tetrises).
//!
//! **Limitations:**
//!
//! - **No formal justification**: Coefficients and formulas lack theoretical or empirical validation
//! - **Unknown optimality**: It's unclear if these formulas effectively capture desired play styles
//! - **Survival vs score balance**: The trade-off between survival and score is not systematically explored
//! - **Ad-hoc design**: Different evaluators use different formulas without consistent design principles
//!
//! The fitness function defines what the AI learns, so improvements to fitness design could
//! significantly impact play quality. See the project documentation (`docs/future-projects.md`)
//! for potential improvements, including systematic fitness function design and multi-objective
//! optimization.
//!
//! # Usage
//!
//! ```rust,no_run
//! use oxidris_evaluator::{
//!     session_evaluator::{DefaultSessionEvaluator, AggroSessionEvaluator},
//!     turn_evaluator::TurnEvaluator,
//! };
//!
//! // Create session evaluator
//! let fitness_fn = AggroSessionEvaluator::new();
//! let session_evaluator = DefaultSessionEvaluator::new(1000, fitness_fn);
//!
//! // Evaluate multiple game sessions
//! // (requires GameField vector - see oxidris_engine::GameField)
//! // let fitness = session_evaluator.play_and_evaluate_sessions(&fields, &turn_evaluator);
//! ```
//!
//! Session evaluators are used by the genetic algorithm (see `oxidris-training` crate)
//! to evolve feature weights that maximize fitness.

use std::{fmt, iter};

use oxidris_engine::{GameField, GameStats};

use crate::{
    placement_analysis::PlacementAnalysis,
    turn_evaluator::{SessionStats, TurnEvaluator},
};

/// Evaluates session statistics to compute fitness scores.
///
/// Implementations define what "good play" means by assigning fitness scores
/// to game session statistics.
pub trait EvaluateSessionStats {
    /// Type of statistics tracked during the session.
    type Stats: SessionStats;

    /// Computes fitness score from session statistics.
    ///
    /// # Arguments
    /// * `field` - Initial game field (for context)
    /// * `stats` - Collected session statistics
    /// * `turn_limit` - Maximum number of turns allowed
    ///
    /// # Returns
    /// Fitness score (higher is better)
    fn evaluate_session_stats(
        &self,
        field: &GameField,
        stats: &Self::Stats,
        turn_limit: usize,
    ) -> f32;
}

/// Evaluates complete game sessions for training.
///
/// Used by the genetic algorithm to compute fitness scores for individuals.
pub trait SessionEvaluator: fmt::Debug + Send + Sync {
    /// Plays and evaluates a single game session.
    fn play_and_evaluate_session(&self, field: &GameField, turn_evaluator: &TurnEvaluator) -> f32;

    /// Plays and evaluates multiple game sessions, returning average fitness.
    fn play_and_evaluate_sessions(
        &self,
        fields: &[GameField],
        turn_evaluator: &TurnEvaluator,
    ) -> f32;
}

/// Default session evaluator implementation.
///
/// Plays game sessions up to a turn limit and evaluates them using a fitness function.
#[derive(Debug)]
pub struct DefaultSessionEvaluator<E> {
    turn_limit: usize,
    evaluator: E,
}

impl<E> DefaultSessionEvaluator<E> {
    /// Creates a new session evaluator.
    ///
    /// # Arguments
    /// * `turn_limit` - Maximum number of turns per game session
    /// * `evaluator` - Fitness function to evaluate session statistics
    pub fn new(turn_limit: usize, evaluator: E) -> Self {
        Self {
            turn_limit,
            evaluator,
        }
    }
}

impl<S, E> SessionEvaluator for DefaultSessionEvaluator<E>
where
    E: EvaluateSessionStats<Stats = S> + fmt::Debug + Send + Sync,
    S: SessionStats,
{
    fn play_and_evaluate_session(&self, field: &GameField, turn_evaluator: &TurnEvaluator) -> f32 {
        let stats = turn_evaluator.play_session(&mut field.clone(), self.turn_limit);
        self.evaluator
            .evaluate_session_stats(field, &stats, self.turn_limit)
    }

    #[expect(clippy::cast_precision_loss)]
    fn play_and_evaluate_sessions(
        &self,
        fields: &[GameField],
        turn_evaluator: &TurnEvaluator,
    ) -> f32 {
        let mut total_fitness = 0.0;
        for field in fields {
            total_fitness += self.play_and_evaluate_session(field, turn_evaluator);
        }
        total_fitness / (fields.len() as f32)
    }
}

/// Default session statistics tracker.
///
/// Tracks game statistics (pieces placed, lines cleared) and worst max height
/// during the session.
#[derive(Debug)]
pub struct DefaultSessionStats {
    game_stats: GameStats,
    worst_max_height: u8,
}

impl SessionStats for DefaultSessionStats {
    fn new() -> Self {
        Self {
            game_stats: GameStats::new(),
            worst_max_height: 0,
        }
    }

    fn complete_piece_drop(&mut self, analysis: &PlacementAnalysis) {
        self.game_stats
            .complete_piece_drop(analysis.cleared_lines());
        let max_height = *analysis
            .board_analysis()
            .column_heights()
            .iter()
            .max()
            .unwrap();
        if max_height > self.worst_max_height {
            self.worst_max_height = max_height;
        }
    }
}

/// Aggressive fitness function: balances survival with line clearing efficiency.
///
/// **Formula:**
/// ```text
/// fitness = survival_bonus + efficiency × survival_ratio - height_penalty
/// ```
///
/// This evaluator encourages the AI to:
/// - Survive as long as possible (quadratic bonus)
/// - Clear lines efficiently, especially multi-line clears
/// - Avoid building risky high stacks
/// - Maintain sustained performance (efficiency scaled by survival)
#[derive(Default, Debug)]
pub struct AggroSessionEvaluator {}

impl AggroSessionEvaluator {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl EvaluateSessionStats for AggroSessionEvaluator {
    type Stats = DefaultSessionStats;

    #[expect(clippy::cast_precision_loss)]
    fn evaluate_session_stats(
        &self,
        _field: &GameField,
        stats: &Self::Stats,
        turn_limit: usize,
    ) -> f32 {
        const LINE_CLEAR_WEIGHT: [u16; 5] = [0, 1, 3, 5, 8];

        let survived = stats.game_stats.completed_pieces() as f32;
        let max_pieces = turn_limit as f32;
        let survived_ratio = survived / max_pieces;
        let survival_bonus = 2.0 * survived_ratio * survived_ratio;
        let weighted_line_count =
            iter::zip(LINE_CLEAR_WEIGHT, stats.game_stats.line_cleared_counter())
                .map(|(w, c)| f32::from(w) * (*c as f32))
                .sum::<f32>();
        let efficiency = weighted_line_count / survived.max(1.0);
        let height_penalty = f32::from(u8::max(stats.worst_max_height, 10) - 10) / 5.0;
        survival_bonus + efficiency * survived_ratio - height_penalty
    }
}

/// Defensive fitness function: prioritizes survival time above all else.
///
/// **Formula:**
/// ```text
/// fitness = survival_bonus + efficiency / 10.0 - height_penalty
/// ```
///
/// This evaluator encourages the AI to:
/// - Maximize survival time (quadratic bonus, same as Aggro)
/// - Clear lines as a secondary objective (10× lower weight)
/// - Tolerate higher stacks (smaller penalty)
/// - Play conservatively to extend survival
#[derive(Default, Debug)]
pub struct DefensiveSessionEvaluator {}

impl DefensiveSessionEvaluator {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl EvaluateSessionStats for DefensiveSessionEvaluator {
    type Stats = DefaultSessionStats;

    #[expect(clippy::cast_precision_loss)]
    fn evaluate_session_stats(
        &self,
        _field: &GameField,
        stats: &Self::Stats,
        turn_limit: usize,
    ) -> f32 {
        let survived = stats.game_stats.completed_pieces() as f32;
        let max_pieces = turn_limit as f32;
        let survived_ratio = survived / max_pieces;
        let survival_bonus = 2.0 * survived_ratio * survived_ratio;
        let line_count = stats.game_stats.total_cleared_lines() as f32;
        let efficiency = line_count / survived.max(1.0);
        let height_penalty = f32::from(stats.worst_max_height) / 20.0;
        survival_bonus + efficiency * survived_ratio - height_penalty
    }
}
