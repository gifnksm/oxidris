use std::iter;

use arrayvec::ArrayVec;
use oxidris_engine::{BitBoard, CompletePieceDropError, GameField, GameStats, Piece};

use crate::{placement_analysis::PlacementAnalysis, placement_evaluator::PlacementEvaluator};

pub trait SessionStats: Sized {
    fn new() -> Self;
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

#[derive(Debug, Clone, Copy)]
pub struct TurnPlan {
    use_hold: bool,
    placement: Piece,
}

impl TurnPlan {
    #[must_use]
    pub fn use_hold(&self) -> bool {
        self.use_hold
    }

    #[must_use]
    pub fn placement(&self) -> Piece {
        self.placement
    }

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

#[derive(Debug)]
pub struct TurnEvaluator {
    placement_evaluator: Box<dyn PlacementEvaluator>,
}

impl TurnEvaluator {
    pub fn new<E>(placement_evaluator: E) -> Self
    where
        E: PlacementEvaluator + 'static,
    {
        Self {
            placement_evaluator: Box::new(placement_evaluator),
        }
    }

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
