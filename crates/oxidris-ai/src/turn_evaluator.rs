use std::iter;

use arrayvec::ArrayVec;
use oxidris_engine::{BitBoard, GameField, Piece};

use crate::PlacementEvaluator;

#[derive(Debug, Clone, Copy)]
pub struct TurnPlan {
    pub use_hold: bool,
    pub placement: Piece,
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
    pub fn select_best_turn(&self, field: &GameField) -> Option<TurnPlan> {
        let mut best_score = f32::MIN;
        let mut best_result = None;

        for turn in available_turns(field).into_iter().flatten() {
            let score = self
                .placement_evaluator
                .evaluate_placement(field.board(), turn.placement);
            if score > best_score {
                best_score = score;
                best_result = Some(turn);
            }
        }

        best_result
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
    let p1 = *field.falling_piece();
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
    piece.left().filter(|moved| !board.is_colliding(moved))
}

fn right(piece: Piece, board: &BitBoard) -> Option<Piece> {
    piece.right().filter(|moved| !board.is_colliding(moved))
}
