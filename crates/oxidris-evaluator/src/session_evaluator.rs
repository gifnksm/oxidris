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
//! Balances line clearing efficiency with height management:
//!
//! ```text
//! fitness = (efficiency + (1 - max_height_penalty) + (1 - peak_max_height_penalty)) / 3
//!
//! where:
//!   efficiency = weighted_lines / max_possible_weighted_lines
//!   weighted_lines = Σ(line_clear_weight[i] × cleared_count[i])
//!   line_clear_weight = [0, 1, 3, 5, 8] for 0-4 line clears
//!   max_height_penalty = avg_squared_height_excess / worst_case_squared_height
//!   peak_max_height_penalty = peak_height_excess² / max_height²
//!   height_excess = max(height - cutoff, 0) where cutoff = 4.0
//! ```
//!
//! **Characteristics:**
//!
//! - Rewards efficient line clearing with exponential weights (4-line Tetris gets 8× value)
//! - Penalizes both average and peak height above cutoff (4.0)
//! - Survival time indirectly penalized: early termination assumes worst-case height for remaining turns
//! - Equal weight given to efficiency, average height quality, and peak height quality
//!
//! ## Defensive Session Evaluator
//!
//! Prioritizes height minimization above all else:
//!
//! ```text
//! fitness = ((1 - max_height_penalty) + (1 - peak_max_height_penalty)) / 2
//!
//! where:
//!   max_height_penalty = avg_squared_height / worst_case_squared_height
//!   peak_max_height_penalty = peak_height² / max_height²
//!   (no cutoff: all height is penalized)
//! ```
//!
//! **Characteristics:**
//!
//! - No efficiency component (line clearing not directly rewarded)
//! - Penalizes all height (cutoff = 0.0) for both average and peak
//! - Survival time indirectly penalized (same mechanism as Aggro)
//! - Focus: minimize height, clear lines only as means to reduce height
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
//! **Current Approach:** The fitness formulas and coefficients (line clear weights `[0,1,3,5,8]`,
//! height cutoffs, quadratic penalties) were chosen manually based on intuition about what "good
//! play" means. The line clear weights encourage multi-line clears (especially 4-line Tetrises),
//! and quadratic height penalties emphasize avoiding dangerous stacks.
//!
//! **Survival Time Consideration:** Survival time is indirectly penalized through a worst-case
//! mechanism: if a game ends before `turn_limit`, remaining turns are counted as having maximum
//! height (20.0), which increases the height penalty. This approach penalizes early termination
//! without requiring explicit survival tracking.
//!
//! **Limitations:**
//!
//! - **No formal justification**: Coefficients and formulas lack theoretical or empirical validation
//! - **Unknown optimality**: It's unclear if these formulas effectively capture desired play styles
//! - **Indirect survival penalty**: Worst-case height assumption may not optimally balance survival vs other objectives
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
//!     session_evaluator::{AggroSessionEvaluator, DefaultSessionEvaluator},
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

use std::{collections::BTreeMap, fmt, iter};

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
    max_height_map: BTreeMap<u8, usize>,
    total_height_map: BTreeMap<u8, usize>,
}

impl SessionStats for DefaultSessionStats {
    fn new() -> Self {
        Self {
            game_stats: GameStats::new(),
            max_height_map: BTreeMap::new(),
            total_height_map: BTreeMap::new(),
        }
    }

    fn complete_piece_drop(&mut self, analysis: &PlacementAnalysis) {
        self.game_stats
            .complete_piece_drop(analysis.cleared_lines());
        *self
            .max_height_map
            .entry(analysis.board_analysis().max_height())
            .or_default() += 1;
        *self
            .total_height_map
            .entry(analysis.board_analysis().total_height())
            .or_default() += 1;
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
        let survived_turns = stats.game_stats.turn() as f32;
        let turn_limit = turn_limit as f32;

        let block_count = turn_limit * 4.0;
        let weighted_line_count =
            iter::zip(LINE_CLEAR_WEIGHT, stats.game_stats.line_cleared_counter())
                .map(|(w, c)| f32::from(w) * (*c as f32))
                .sum::<f32>();
        let max_line_score = 8.0 * block_count / 40.0;
        let efficiency = weighted_line_count / max_line_score;

        let max_height_max: f32 = 20.0;
        let max_height_cutoff = 4.0;
        let mut max_height_square_sum = stats
            .max_height_map
            .iter()
            .map(|(h, c)| {
                let h = f32::from(*h) - max_height_cutoff;
                if h < 0.0 {
                    0.0
                } else {
                    h.powi(2) * (*c as f32)
                }
            })
            .sum::<f32>();
        if survived_turns < turn_limit {
            // Penalize unfinished games by assuming max height for remaining turns
            let remaining_turns = turn_limit - survived_turns;
            max_height_square_sum += (max_height_max - max_height_cutoff).powi(2) * remaining_turns;
        }

        let worst_max_height = (max_height_max - max_height_cutoff).powi(2) * turn_limit;
        let max_height_penalty = max_height_square_sum / worst_max_height;

        let peak_max_height = stats
            .max_height_map
            .last_key_value()
            .map_or(0.0, |(h, _c)| {
                let h = f32::from(*h) - max_height_cutoff;
                if h < 0.0 { 0.0 } else { h }
            });
        let peak_max_height_penalty = peak_max_height.powi(2) / max_height_max.powi(2);

        (efficiency + (1.0 - max_height_penalty) + (1.0 - peak_max_height_penalty)) / 3.0
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

    #[expect(clippy::cast_precision_loss, clippy::manual_midpoint)]
    fn evaluate_session_stats(
        &self,
        _field: &GameField,
        stats: &Self::Stats,
        turn_limit: usize,
    ) -> f32 {
        let survived_turns = stats.game_stats.turn() as f32;
        let turn_limit = turn_limit as f32;

        let max_height_max: f32 = 20.0;
        let max_height_cutoff = 0.0;
        let mut max_height_square_sum = stats
            .max_height_map
            .iter()
            .map(|(h, c)| {
                let h = f32::from(*h) - max_height_cutoff;
                if h < 0.0 {
                    0.0
                } else {
                    h.powi(2) * (*c as f32)
                }
            })
            .sum::<f32>();
        if survived_turns < turn_limit {
            // Penalize unfinished games by assuming max height for remaining turns
            let remaining_turns = turn_limit - survived_turns;
            max_height_square_sum += (max_height_max - max_height_cutoff).powi(2) * remaining_turns;
        }

        let worst_max_height = (max_height_max - max_height_cutoff).powi(2) * turn_limit;
        let max_height_penalty = max_height_square_sum / worst_max_height;

        let peak_max_height = stats
            .max_height_map
            .last_key_value()
            .map_or(0.0, |(h, _c)| {
                let h = f32::from(*h) - max_height_cutoff;
                if h < 0.0 { 0.0 } else { h }
            });
        let peak_max_height_penalty = peak_max_height.powi(2) / max_height_max.powi(2);

        ((1.0 - max_height_penalty) + (1.0 - peak_max_height_penalty)) / 2.0
    }
}
