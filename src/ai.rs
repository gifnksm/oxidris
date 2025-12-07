use std::iter;

use arrayvec::ArrayVec;

use crate::{
    field::Field,
    ga::{GenoSeq, GenomeKind},
    game::GameCore,
    mino::Mino,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Move {
    pub(crate) is_hold_used: bool,
    pub(crate) mino: Mino,
}

pub(crate) fn eval(game: &GameCore, weight: GenoSeq) -> Option<(Move, GameCore)> {
    let mut best_score = f64::MIN;
    let mut best_result = None;
    let init_lines_cleared = game.cleared_lines();

    for (game, moves) in available_moves(game.clone()) {
        for mv in moves {
            let mut game = game.clone();
            game.set_falling_mino_unchecked(mv.mino);
            if game.complete_mino_drop().is_err() {
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
                    next.set_falling_mino_unchecked(next_mv.mino);
                    let game_over = next.complete_mino_drop().is_err();
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

fn available_moves(mut game: GameCore) -> ArrayVec<(GameCore, impl Iterator<Item = Move>), 2> {
    let mut result = ArrayVec::new();
    let moves = available_mino_moves(&game);
    result.push((game.clone(), moves));
    if game.try_hold().is_ok() {
        let moves = available_mino_moves(&game);
        result.push((game, moves));
    }
    result
}

fn available_mino_moves(game: &GameCore) -> impl Iterator<Item = Move> + use<> {
    let field = game.field().clone();
    let is_hold_used = game.is_hold_used();
    let rotations = game.falling_mino().super_rotations(&field).into_iter();
    rotations.flat_map(move |mino| {
        iter::once(mino)
            .chain(iter::successors(move_left(&mino, &field), |m| {
                move_left(m, &field)
            }))
            .chain(iter::successors(move_right(&mino, &field), |m| {
                move_right(m, &field)
            }))
            .map(|mino| mino.simulate_drop_position(&field))
            .map(move |mino| Move { is_hold_used, mino })
            .collect::<ArrayVec<_, { Field::BLOCKS_WIDTH }>>()
    })
}

fn move_left(mino: &Mino, field: &Field) -> Option<Mino> {
    mino.left().filter(|moved| !field.is_colliding(moved))
}

fn move_right(mino: &Mino, field: &Field) -> Option<Mino> {
    mino.right().filter(|moved| !field.is_colliding(moved))
}

fn compute_score(
    weight: GenoSeq,
    game: &GameCore,
    game_over: bool,
    init_lines_cleared: usize,
) -> f64 {
    if game_over {
        return 0.0;
    }

    let lines_cleared = u16::try_from(game.cleared_lines() - init_lines_cleared).unwrap();
    let height_max = field_height_max(game.field());
    let height_diff = diff_in_height(game.field());
    let dead_space = dead_space_count(game.field());

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

fn field_height(field: &Field, x: usize) -> u16 {
    let height = field
        .block_rows()
        .enumerate()
        .find(|(_y, row)| !row[x].is_empty())
        .map_or(0, |(y, _row)| Field::BLOCKS_HEIGHT - y);

    u16::try_from(height).unwrap()
}

fn field_height_max(field: &Field) -> u16 {
    let max = (0..Field::BLOCKS_HEIGHT)
        .find(|&y| field.block_row(y).iter().any(|block| !block.is_empty()))
        .map_or(0, |y| Field::BLOCKS_HEIGHT - y);

    u16::try_from(max).unwrap()
}

fn diff_in_height(field: &Field) -> u16 {
    let mut diff = 0;
    let mut top = [0; Field::BLOCKS_WIDTH];

    for (x, top) in top.iter_mut().enumerate() {
        *top = field_height(field, x);
    }
    for i in 0..top.len() - 1 {
        diff += top[i].abs_diff(top[i + 1]);
    }

    diff
}

fn dead_space_count(field: &Field) -> u16 {
    let count = (0..Field::BLOCKS_WIDTH)
        .map(|x| {
            field
                .block_rows()
                .map(|row| row[x])
                .skip_while(|block| block.is_empty())
                .filter(|block| block.is_empty())
                .count()
        })
        .sum::<usize>();

    u16::try_from(count).unwrap()
}
