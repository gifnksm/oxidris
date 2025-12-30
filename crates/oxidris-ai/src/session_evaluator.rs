use std::{fmt, iter};

use oxidris_engine::{GameField, GameStats};

use crate::turn_evaluator::TurnEvaluator;

pub trait SessionEvaluator: fmt::Debug + Send + Sync {
    fn turn_limit(&self) -> usize;
    fn evaluate_session_stats(&self, field: &GameField, stats: &GameStats) -> f32;

    fn play_and_evaluate_session(&self, field: &GameField, turn_evaluator: &TurnEvaluator) -> f32 {
        let stats = turn_evaluator.play_session(&mut field.clone(), self.turn_limit());
        self.evaluate_session_stats(field, &stats)
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
pub struct AggroSessionEvaluator {
    turn_limit: usize,
}

impl AggroSessionEvaluator {
    #[must_use]
    pub fn new(turn_limit: usize) -> Self {
        Self { turn_limit }
    }
}

impl SessionEvaluator for AggroSessionEvaluator {
    fn turn_limit(&self) -> usize {
        self.turn_limit
    }

    #[expect(clippy::cast_precision_loss)]
    fn evaluate_session_stats(&self, _field: &GameField, stats: &GameStats) -> f32 {
        const LINE_CLEAR_WEIGHT: [u16; 5] = [0, 1, 3, 5, 8];

        let survived = stats.completed_pieces() as f32;
        let max_pieces = self.turn_limit as f32;
        let survived_ratio = survived / max_pieces;
        let survival_bonus = 2.0 * survived_ratio * survived_ratio;
        let weighted_line_count = iter::zip(LINE_CLEAR_WEIGHT, stats.line_cleared_counter())
            .map(|(w, c)| f32::from(w) * (*c as f32))
            .sum::<f32>();
        let efficiency = weighted_line_count / survived.max(1.0);
        survival_bonus + efficiency * survived_ratio
    }
}

#[derive(Debug)]
pub struct DefensiveSessionEvaluator {
    turn_limit: usize,
}

impl DefensiveSessionEvaluator {
    #[must_use]
    pub fn new(turn_limit: usize) -> Self {
        Self { turn_limit }
    }
}

impl SessionEvaluator for DefensiveSessionEvaluator {
    fn turn_limit(&self) -> usize {
        self.turn_limit
    }

    #[expect(clippy::cast_precision_loss)]
    fn evaluate_session_stats(&self, field: &GameField, stats: &GameStats) -> f32 {
        let survived = stats.completed_pieces() as f32;
        let max_pieces = self.turn_limit as f32;
        let survived_ratio = survived / max_pieces;
        let survival_bonus = 2.0 * survived_ratio * survived_ratio;
        let line_count = stats.total_cleared_lines() as f32;
        let efficiency = line_count / survived.max(1.0);
        let height_penalty = (field.board().max_height() as f32) / 20.0;
        survival_bonus + efficiency * survived_ratio - height_penalty
    }
}
