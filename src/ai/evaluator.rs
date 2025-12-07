use std::iter;

use arrayvec::ArrayVec;

use crate::{
    ai::genetic::{GenoSeq, GenomeKind},
    core::{piece::Piece, render_board::RenderBoard},
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

fn move_left(piece: &Piece, board: &RenderBoard) -> Option<Piece> {
    piece.left().filter(|moved| !board.is_colliding(moved))
}

fn move_right(piece: &Piece, board: &RenderBoard) -> Option<Piece> {
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

    let lines_cleared = u16::try_from(game.cleared_lines() - init_lines_cleared).unwrap();
    let height_max = compute_height_max(game.board());
    let height_diff = compute_height_diff(game.board());
    let dead_space = compute_dead_space(game.board());

    let mut lines_cleared = normalize(f64::from(lines_cleared), 0.0, 8.0);
    let mut height_max = 1.0 - normalize(f64::from(height_max), 0.0, 20.0);
    let mut height_diff = 1.0 - normalize(f64::from(height_diff), 0.0, 200.0);
    let mut dead_space = 1.0 - normalize(f64::from(dead_space), 0.0, 200.0);

    lines_cleared *= f64::from(weight[GenomeKind::LinesCleared]);
    height_max *= f64::from(weight[GenomeKind::HeightMax]);
    height_diff *= f64::from(weight[GenomeKind::HeightDiff]);
    dead_space *= f64::from(weight[GenomeKind::DeadSpace]);

    lines_cleared + height_max + height_diff + dead_space
}

fn normalize(value: f64, min: f64, max: f64) -> f64 {
    (value - min) / (max - min)
}

fn column_height(board: &RenderBoard, x: usize) -> u16 {
    let height = board
        .playable_rows()
        .enumerate()
        .find(|(_y, row)| !row[x].is_empty())
        .map_or(0, |(y, _row)| RenderBoard::PLAYABLE_HEIGHT - y);

    u16::try_from(height).unwrap()
}

fn compute_height_max(board: &RenderBoard) -> u16 {
    let max = (0..RenderBoard::PLAYABLE_HEIGHT)
        .find(|&y| board.playable_row(y).iter().any(|cell| !cell.is_empty()))
        .map_or(0, |y| RenderBoard::PLAYABLE_HEIGHT - y);

    u16::try_from(max).unwrap()
}

fn compute_height_diff(board: &RenderBoard) -> u16 {
    let mut diff = 0;
    let mut top = [0; RenderBoard::PLAYABLE_WIDTH];

    for (x, top) in top.iter_mut().enumerate() {
        *top = column_height(board, x);
    }
    for i in 0..top.len() - 1 {
        diff += top[i].abs_diff(top[i + 1]);
    }

    diff
}

fn compute_dead_space(board: &RenderBoard) -> u16 {
    let count = (0..RenderBoard::PLAYABLE_WIDTH)
        .map(|x| {
            board
                .playable_rows()
                .map(|row| row[x])
                .skip_while(|cell| cell.is_empty())
                .filter(|cell| cell.is_empty())
                .count()
        })
        .sum::<usize>();

    u16::try_from(count).unwrap()
}
