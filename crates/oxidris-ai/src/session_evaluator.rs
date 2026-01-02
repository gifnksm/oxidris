use std::{fmt, iter};

use oxidris_engine::{GameField, GameStats};

use crate::{
    placement_analysis::PlacementAnalysis,
    turn_evaluator::{SessionStats, TurnEvaluator},
};

pub trait EvaluateSessionStats {
    type Stats: SessionStats;
    fn evaluate_session_stats(
        &self,
        field: &GameField,
        stats: &Self::Stats,
        turn_limit: usize,
    ) -> f32;
}

pub trait SessionEvaluator: fmt::Debug + Send + Sync {
    fn play_and_evaluate_session(&self, field: &GameField, turn_evaluator: &TurnEvaluator) -> f32;
    fn play_and_evaluate_sessions(
        &self,
        fields: &[GameField],
        turn_evaluator: &TurnEvaluator,
    ) -> f32;
}

#[derive(Debug)]
pub struct DefaultSessionEvaluator<E> {
    turn_limit: usize,
    evaluator: E,
}

impl<E> DefaultSessionEvaluator<E> {
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
