use std::iter;

use arrayvec::ArrayVec;

use super::metrics::{self, METRIC_COUNT};
use super::weights::WeightSet;
use crate::core::bit_board::BitBoard;
use crate::core::piece::Piece;
use crate::engine::state::GameState;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Move {
    pub(crate) is_hold_used: bool,
    pub(crate) piece: Piece,
}

#[derive(Debug, Clone)]
pub(crate) struct Evaluator {
    weights: WeightSet<{ METRIC_COUNT }>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new(WeightSet::BEST)
    }
}

impl Evaluator {
    pub(crate) fn new(weights: WeightSet<{ METRIC_COUNT }>) -> Self {
        Self { weights }
    }

    #[inline]
    pub(crate) fn score(&self, init: &GameState, game: &GameState, game_over: bool) -> f32 {
        if game_over {
            return 0.0;
        }

        let metrics = metrics::measure(init, game);
        iter::zip(metrics, self.weights.0).map(|(m, w)| m * w).sum()
    }

    pub(crate) fn select_move(&self, game: &GameState) -> Option<(Move, GameState)> {
        let mut best_score = f32::MIN;
        let mut best_result = None;
        let init = game;

        for (game, moves) in available_moves(game.clone()) {
            for mv in moves {
                let mut game = game.clone();
                game.set_falling_piece_unchecked(mv.piece);
                let game_over = game.complete_piece_drop().is_err();
                let score = self.score(init, &game, game_over);
                if score > best_score {
                    best_score = score;
                    best_result = Some((mv, game.clone()));
                }
            }
        }

        best_result
    }
}

fn available_moves(mut game: GameState) -> ArrayVec<(GameState, impl Iterator<Item = Move>), 2> {
    let mut result = ArrayVec::new();
    let moves = available_piece_moves(&game);
    result.push((game.clone(), moves));
    if game.try_hold().is_ok() {
        let moves = available_piece_moves(&game);
        result.push((game, moves));
    }
    result
}

fn available_piece_moves(game: &GameState) -> impl Iterator<Item = Move> + use<> {
    let board = game.board().clone();
    let is_hold_used = game.is_hold_used();
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
            .map(move |piece| Move {
                is_hold_used,
                piece,
            })
            .collect::<ArrayVec<_, { BitBoard::PLAYABLE_WIDTH }>>()
    })
}

fn move_left(piece: &Piece, board: &BitBoard) -> Option<Piece> {
    piece.left().filter(|moved| !board.is_colliding(moved))
}

fn move_right(piece: &Piece, board: &BitBoard) -> Option<Piece> {
    piece.right().filter(|moved| !board.is_colliding(moved))
}
