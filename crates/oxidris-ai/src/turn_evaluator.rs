use std::iter;

use arrayvec::ArrayVec;
use oxidris_engine::{BitBoard, GameState, Piece};

use super::metrics::Metrics;
use super::weights::WeightSet;
use crate::AiType;

#[derive(Debug, Clone, Copy)]
pub struct TurnPlan {
    pub use_hold: bool,
    pub placement: Piece,
}

#[derive(Debug, Clone)]
pub struct TurnEvaluator {
    weights: WeightSet,
}

impl TurnEvaluator {
    #[must_use]
    pub fn aggro() -> Self {
        Self {
            weights: WeightSet::AGGRO,
        }
    }

    #[must_use]
    pub fn defensive() -> Self {
        Self {
            weights: WeightSet::DEFENSIVE,
        }
    }

    #[must_use]
    pub fn by_ai_type(ai: AiType) -> Self {
        match ai {
            AiType::Aggro => Self::aggro(),
            AiType::Defensive => Self::defensive(),
        }
    }

    #[must_use]
    pub fn new(weights: WeightSet) -> Self {
        Self { weights }
    }

    #[inline]
    fn score(&self, board: &BitBoard, placement: Piece) -> f32 {
        let metrics = Metrics::measure(board, placement);
        iter::zip(metrics.to_array(), self.weights.to_array())
            .map(|(m, w)| m * w)
            .sum()
    }

    #[must_use]
    pub fn select_best_turn(&self, game: &GameState) -> Option<TurnPlan> {
        let mut best_score = f32::MIN;
        let mut best_result = None;

        for (game, turns) in available_turns(game.clone()) {
            for turn in turns {
                let score = self.score(game.board(), turn.placement);
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
    mut game: GameState,
) -> ArrayVec<(GameState, impl Iterator<Item = TurnPlan>), 2> {
    let mut result = ArrayVec::new();
    let placement2turn = |use_hold| {
        move |placement| TurnPlan {
            use_hold,
            placement,
        }
    };

    let turns = available_placement(&game);
    result.push((game.clone(), turns.map(placement2turn(false))));
    if game.try_hold().is_ok() {
        let turns = available_placement(&game).map(placement2turn(true));
        result.push((game, turns));
    }
    result
}

fn available_placement(game: &GameState) -> impl Iterator<Item = Piece> + use<> {
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
