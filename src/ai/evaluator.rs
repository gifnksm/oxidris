use std::iter;

use arrayvec::ArrayVec;

use crate::{
    ai::genetic::{GenoSeq, GenomeKind},
    core::{
        bit_board::{BitBoard, SENTINEL_MARGIN_LEFT},
        piece::Piece,
        render_board::RenderBoard,
    },
    engine::state::GameState,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Move {
    pub(crate) is_hold_used: bool,
    pub(crate) piece: Piece,
}

pub(crate) fn eval(game: &GameState, weight: GenoSeq) -> Option<(Move, GameState)> {
    let mut best_score = f64::MIN;
    let mut best_result = None;
    let init_lines_cleared = game.cleared_lines();

    for (game, moves) in available_moves(game.clone()) {
        for mv in moves {
            let mut game = game.clone();
            game.set_falling_piece_unchecked(mv.piece);
            if game.complete_piece_drop().is_err() {
                let score = compute_score(weight, &game, true, init_lines_cleared);
                if score > best_score {
                    best_score = score;
                    best_result = Some((mv, game.clone()));
                }
                continue;
            }

            for (next, moves) in available_moves(game.clone()) {
                for next_mv in moves {
                    let mut next = next.clone();
                    next.set_falling_piece_unchecked(next_mv.piece);
                    let game_over = next.complete_piece_drop().is_err();
                    let score = compute_score(weight, &next, game_over, init_lines_cleared);
                    if score > best_score {
                        best_score = score;
                        best_result = Some((mv, game.clone()));
                    }
                }
            }
        }
    }

    best_result
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
            .collect::<ArrayVec<_, { RenderBoard::PLAYABLE_WIDTH }>>()
    })
}

fn move_left(piece: &Piece, board: &BitBoard) -> Option<Piece> {
    piece.left().filter(|moved| !board.is_colliding(moved))
}

fn move_right(piece: &Piece, board: &BitBoard) -> Option<Piece> {
    piece.right().filter(|moved| !board.is_colliding(moved))
}

fn compute_score(
    weight: GenoSeq,
    game: &GameState,
    game_over: bool,
    init_lines_cleared: usize,
) -> f64 {
    if game_over {
        return 0.0;
    }

    let lines_cleared =
        f64::from(u16::try_from(game.cleared_lines() - init_lines_cleared).unwrap());
    let height_info = HeightInfo::compute(game.board());

    let lines_cleared_norm = normalize(lines_cleared, 0.0, 8.0);
    let max_height_norm = height_info.normalized_max_height();
    let height_diff_norm = height_info.normalized_height_diff();
    let dead_space_norm = height_info.normalized_dead_space();

    let mut lines_cleared = lines_cleared_norm;
    let mut height_max = 1.0 - max_height_norm;
    let mut height_diff = 1.0 - height_diff_norm;
    let mut dead_space = 1.0 - dead_space_norm;

    lines_cleared *= f64::from(weight[GenomeKind::LinesCleared]);
    height_max *= f64::from(weight[GenomeKind::HeightMax]);
    height_diff *= f64::from(weight[GenomeKind::HeightDiff]);
    dead_space *= f64::from(weight[GenomeKind::DeadSpace]);

    lines_cleared + height_max + height_diff + dead_space
}

fn normalize(value: impl Into<f64>, min: impl Into<f64>, max: impl Into<f64>) -> f64 {
    let min = min.into();
    let max = max.into();
    let value = value.into();
    assert!(min <= value && value <= max);
    let value = value.clamp(min, max);
    (value - min) / (max - min)
}

#[derive(Debug, Clone, Copy)]
struct HeightInfo {
    heights: [u8; BitBoard::PLAYABLE_WIDTH],
    occupied: [u8; BitBoard::PLAYABLE_WIDTH],
}

impl HeightInfo {
    fn compute(board: &BitBoard) -> Self {
        let mut heights = [0; BitBoard::PLAYABLE_WIDTH];
        let mut occupied = [0; BitBoard::PLAYABLE_WIDTH];
        for i in 0..BitBoard::PLAYABLE_WIDTH {
            let x = SENTINEL_MARGIN_LEFT + i;
            let min_y = board
                .playable_rows()
                .enumerate()
                .find(|(_y, row)| row.is_cell_occupied(x));
            let Some((min_y, _)) = min_y else {
                continue;
            };
            heights[i] = u8::try_from(BitBoard::PLAYABLE_HEIGHT - min_y).unwrap();
            occupied[i] = 1;
            for y in min_y + 1..BitBoard::PLAYABLE_HEIGHT {
                let row = board.playable_row(y);
                if row.is_cell_occupied(x) {
                    occupied[i] += 1;
                }
            }
        }
        Self { heights, occupied }
    }

    fn normalized_max_height(&self) -> f64 {
        const MIN: u8 = 0;
        #[expect(clippy::cast_possible_truncation)]
        const MAX: u8 = BitBoard::PLAYABLE_HEIGHT as u8;
        let height = *self.heights.iter().max().unwrap();
        normalize(height, MIN, MAX)
    }

    fn normalized_height_diff(&self) -> f64 {
        const MIN: u8 = 0;
        #[expect(clippy::cast_possible_truncation)]
        const MAX: u8 = BitBoard::PLAYABLE_HEIGHT as u8 * BitBoard::PLAYABLE_WIDTH as u8;
        let diff = self
            .heights
            .iter()
            .zip(&self.heights[1..])
            .map(|(&a, &b)| a.abs_diff(b))
            .sum::<u8>();
        normalize(diff, MIN, MAX)
    }

    fn normalized_dead_space(&self) -> f64 {
        const MIN: u8 = 0;
        #[expect(clippy::cast_possible_truncation)]
        const MAX: u8 = BitBoard::PLAYABLE_HEIGHT as u8 * BitBoard::PLAYABLE_WIDTH as u8;
        let dead_space = iter::zip(&self.heights, &self.occupied)
            .map(|(&h, &occ)| h - occ)
            .sum::<u8>();
        normalize(dead_space, MIN, MAX)
    }
}
