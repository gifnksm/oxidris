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

        for (field, turns) in available_turns(field.clone()) {
            for turn in turns {
                let score = self
                    .placement_evaluator
                    .evaluate_placement(field.board(), turn.placement);
                if score > best_score {
                    best_score = score;
                    best_result = Some(turn);
                }
            }
        }

        best_result
    }
}

fn available_turns(
    mut field: GameField,
) -> ArrayVec<(GameField, impl Iterator<Item = TurnPlan>), 2> {
    let mut result = ArrayVec::new();
    let placement2turn = |use_hold| {
        move |placement| TurnPlan {
            use_hold,
            placement,
        }
    };

    let turns = available_placement(&field);
    result.push((field.clone(), turns.map(placement2turn(false))));
    if field.try_hold().is_ok() {
        let turns = available_placement(&field).map(placement2turn(true));
        result.push((field, turns));
    }
    result
}

fn available_placement(game: &GameField) -> impl Iterator<Item = Piece> + use<> {
    let board = game.board().clone();
    let rotations = game.falling_piece().super_rotations(&board).into_iter();
    rotations.flat_map(move |piece| {
        iter::once(piece)
            .chain(iter::successors(move_left(&piece, &board), |p| {
                move_left(p, &board)
            }))
            .chain(iter::successors(move_right(&piece, &board), |p| {
                move_right(p, &board)
            }))
            .map(|piece| piece.simulate_drop_position(&board))
            .collect::<ArrayVec<_, { BitBoard::PLAYABLE_WIDTH }>>()
    })
}

fn move_left(piece: &Piece, board: &BitBoard) -> Option<Piece> {
    piece.left().filter(|moved| !board.is_colliding(moved))
}

fn move_right(piece: &Piece, board: &BitBoard) -> Option<Piece> {
    piece.right().filter(|moved| !board.is_colliding(moved))
}
