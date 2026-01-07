//! Turn evaluation: selecting the best piece placement for the current turn.
//!
//! This module implements the second level of the evaluator architecture: choosing which
//! placement to make for the current game turn by evaluating all possible options and
//! selecting the one with the highest score.
//!
//! # How It Works
//!
//! Turn evaluation follows these steps:
//!
//! 1. **Enumerate Placements** - Generate all valid placements for current and hold pieces
//! 2. **Score Each Placement** - Use placement evaluator to score each option
//! 3. **Select Best** - Choose the placement with the highest score
//!
//! # Turn Plan
//!
//! A [`TurnPlan`] specifies the complete action for a turn:
//! - Whether to use hold (swap current piece with held piece)
//! - Final piece placement (position and rotation)
//!
//! The turn evaluator considers both the current falling piece and the hold piece,
//! evaluating all valid placements for each.
//!
//! # Design: Greedy One-Step Lookahead
//!
//! The [`TurnEvaluator`] uses a greedy approach: it only looks at the immediate next
//! placement, not future turns. This makes it fast but potentially suboptimal for
//! multi-step strategies.
//!
//! **Advantages:**
//!
//! - Very fast (evaluates ~100-200 placements per turn)
//! - No exponential search space
//! - Good enough for most gameplay
//!
//! **Limitations:**
//!
//! - No multi-turn planning (e.g., setting up T-spins or back-to-back Tetrises)
//! - Purely reactive, not strategic
//!
//! # Usage
//!
//! ```rust,no_run
//! use oxidris_evaluator::{
//!     placement_evaluator::FeatureBasedPlacementEvaluator, turn_evaluator::TurnEvaluator,
//! };
//! # let features = todo!(); // Build features with normalization parameters
//! # let weights = todo!(); // Load trained weights
//!
//! // Create placement evaluator
//! let placement_evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
//!
//! // Create turn evaluator
//! let turn_evaluator = TurnEvaluator::new(Box::new(placement_evaluator));
//!
//! // Select best turn for current game state
//! // (requires GameField - see oxidris_engine::GameField for details)
//! // if let Some((turn_plan, analysis)) = turn_evaluator.select_best_turn(&field) {
//! //     // Apply the selected turn
//! //     turn_plan.apply(&analysis, &mut field, &mut stats);
//! // }
//! ```

use std::iter;

use arrayvec::ArrayVec;
use oxidris_engine::{BitBoard, CompletePieceDropError, GameField, GameStats, Piece};

use crate::{placement_analysis::PlacementAnalysis, placement_evaluator::PlacementEvaluator};

/// Statistics tracking for game sessions.
///
/// This trait allows different statistics to be collected during gameplay,
/// used by session evaluators to compute fitness scores during training.
pub trait SessionStats: Sized {
    /// Creates a new statistics tracker.
    fn new() -> Self;

    /// Updates statistics after a piece is placed.
    fn complete_piece_drop(&mut self, analysis: &PlacementAnalysis);
}

impl SessionStats for GameStats {
    fn new() -> Self {
        GameStats::new()
    }

    fn complete_piece_drop(&mut self, analysis: &PlacementAnalysis) {
        self.complete_piece_drop(analysis.cleared_lines());
    }
}

/// A complete action plan for a single turn.
///
/// Specifies whether to use hold and where to place the resulting piece.
#[derive(Debug, Clone, Copy)]
pub struct TurnPlan {
    use_hold: bool,
    placement: Piece,
}

impl TurnPlan {
    /// Returns whether this turn plan uses the hold system.
    #[must_use]
    pub fn use_hold(&self) -> bool {
        self.use_hold
    }

    /// Returns the final piece placement for this turn.
    #[must_use]
    pub fn placement(&self) -> Piece {
        self.placement
    }

    /// Applies this turn plan to the game field.
    ///
    /// # Arguments
    ///
    /// * `analysis` - Placement analysis for the chosen placement
    /// * `field` - Game field to modify
    /// * `stats` - Statistics tracker to update
    ///
    /// # Returns
    ///
    /// Tuple of (lines cleared, result) where result indicates if the game ended
    pub fn apply<S>(
        &self,
        analysis: &PlacementAnalysis,
        field: &mut GameField,
        stats: &mut S,
    ) -> (usize, Result<(), CompletePieceDropError>)
    where
        S: SessionStats,
    {
        if self.use_hold {
            field.try_hold().unwrap();
        }
        assert_eq!(field.falling_piece().kind(), self.placement.kind());
        field.set_falling_piece_unchecked(self.placement);
        let (cleared_lines, result) = field.complete_piece_drop();
        stats.complete_piece_drop(analysis);
        (cleared_lines, result)
    }
}

/// Evaluates and selects the best placement for the current turn.
///
/// Uses a placement evaluator to score all possible placements and selects
/// the one with the highest score.
#[derive(Debug)]
pub struct TurnEvaluator<'a> {
    placement_evaluator: Box<dyn PlacementEvaluator + 'a>,
}

impl<'a> TurnEvaluator<'a> {
    /// Creates a new turn evaluator with the given placement evaluator.
    #[must_use]
    pub fn new(placement_evaluator: Box<dyn PlacementEvaluator + 'a>) -> Self {
        Self {
            placement_evaluator,
        }
    }

    /// Selects the best turn for the current game state.
    ///
    /// Evaluates all possible placements (with and without hold) and returns
    /// the one with the highest score according to the placement evaluator.
    ///
    /// # Arguments
    /// * `field` - Current game field state
    ///
    /// # Returns
    /// `Some((turn_plan, analysis))` if a valid placement exists, `None` if game over
    #[must_use]
    pub fn select_best_turn(&self, field: &GameField) -> Option<(TurnPlan, PlacementAnalysis)> {
        let mut best_score = f32::MIN;
        let mut best_result = None;

        for turn in available_turns(field).into_iter().flatten() {
            let analysis = PlacementAnalysis::from_board(field.board(), turn.placement());
            let score = self.placement_evaluator.evaluate_placement(&analysis);
            if score > best_score {
                best_score = score;
                best_result = Some((turn, analysis));
            }
        }

        best_result
    }

    #[must_use]
    pub fn play_session<S>(&self, field: &mut GameField, turn_limit: usize) -> S
    where
        S: SessionStats,
    {
        let mut stats = S::new();
        for _ in 0..turn_limit {
            let Some((turn, analysis)) = self.select_best_turn(field) else {
                return stats;
            };
            let (_cleared_lines, result) = turn.apply(&analysis, field, &mut stats);
            if result.is_err() {
                break;
            }
        }
        stats
    }
}

fn available_turns(field: &GameField) -> ArrayVec<impl Iterator<Item = TurnPlan>, 2> {
    let mut result = ArrayVec::new();
    let placement2turn = |use_hold| {
        move |placement| TurnPlan {
            use_hold,
            placement,
        }
    };

    let board = field.board();
    let p1 = field.falling_piece();
    result.push(available_placement(p1, board).map(placement2turn(false)));

    if field.can_hold() {
        let p2 = field.peek_falling_piece_after_hold();
        result.push(available_placement(p2, board).map(placement2turn(true)));
    }

    result
}

fn available_placement(piece: Piece, board: &BitBoard) -> impl Iterator<Item = Piece> + use<'_> {
    piece
        .super_rotations(board)
        .into_iter()
        .flat_map(move |p| {
            iter::once(p)
                .chain(iter::successors(left(p, board), |p| left(*p, board)))
                .chain(iter::successors(right(p, board), |p| right(*p, board)))
        })
        .map(|piece| piece.simulate_drop_position(board))
}

fn left(piece: Piece, board: &BitBoard) -> Option<Piece> {
    piece.left().filter(|moved| !board.is_colliding(*moved))
}

fn right(piece: Piece, board: &BitBoard) -> Option<Piece> {
    piece.right().filter(|moved| !board.is_colliding(*moved))
}
