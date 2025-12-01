use std::iter;

use crate::{
    field::Field,
    ga::{GenoSeq, GenomeKind},
    game::{DropResult, Game},
};

pub(crate) fn eval(game: &Game, weight: GenoSeq) -> (Game, bool) {
    let mut elite = ((game.clone(), true), f64::MIN);

    for mut game in next_candidates(game) {
        let result = game.hard_drop_and_complete();
        let DropResult::Success {
            lines_cleared: pre_line,
        } = result
        else {
            let score = compute_score(&game, weight, true, 0);
            if elite.1 < score {
                elite.0 = (game, true);
                elite.1 = score;
            }
            continue;
        };

        for mut next in next_candidates(&game) {
            let (line, gameover) = match next.hard_drop_and_complete() {
                DropResult::Success {
                    lines_cleared: line,
                } => (line, false),
                DropResult::GameOver => (0, true),
            };
            let score = compute_score(&next, weight, gameover, pre_line + line);
            if elite.1 < score {
                elite.0 = (game.clone(), gameover);
                elite.1 = score;
            }
        }
    }

    elite.0
}

fn next_candidates(game: &Game) -> impl Iterator<Item = Game> + use<> {
    (iter::once(game.clone()).chain(hold(game))).flat_map(|game| {
        {
            iter::once(game.clone())
                .chain(iter::successors(rotate_right(&game), rotate_right).take(3))
        }
        .flat_map(|game| {
            iter::once(game.clone())
                .chain(iter::successors(move_left(&game), move_left))
                .chain(iter::successors(move_right(&game), move_right))
        })
    })
}

fn compute_score(game: &Game, weight: GenoSeq, gameover: bool, line: usize) -> f64 {
    if gameover {
        return 0.0;
    }

    let line = u16::try_from(line).unwrap();
    let height_max = field_height_max(game);
    let height_diff = diff_in_height(game);
    let dead_space = dead_space_count(game);

    let mut line = normalization(f64::from(line), 0.0, 8.0);
    let mut height_max = 1.0 - normalization(f64::from(height_max), 0.0, 20.0);
    let mut height_diff = 1.0 - normalization(f64::from(height_diff), 0.0, 200.0);
    let mut dead_space = 1.0 - normalization(f64::from(dead_space), 0.0, 200.0);

    line *= f64::from(weight[GenomeKind::Line]);
    height_max *= f64::from(weight[GenomeKind::HeightMax]);
    height_diff *= f64::from(weight[GenomeKind::HeightDiff]);
    dead_space *= f64::from(weight[GenomeKind::DeadSpace]);

    line + height_max + height_diff + dead_space
}

fn hold(game: &Game) -> Option<Game> {
    let mut game = game.clone();
    game.try_hold().ok()?;
    Some(game)
}

fn rotate_right(game: &Game) -> Option<Game> {
    let mut game = game.clone();
    game.try_rotate_right().ok()?;
    Some(game)
}

fn move_left(game: &Game) -> Option<Game> {
    let mut game = game.clone();
    if game.try_move_left().is_err() {
        game.try_soft_drop().ok()?;
        game.try_move_left().ok()?;
    }
    Some(game)
}

fn move_right(game: &Game) -> Option<Game> {
    let mut game = game.clone();
    if game.try_move_right().is_err() {
        game.try_soft_drop().ok()?;
        game.try_move_right().ok()?;
    }
    Some(game)
}

fn normalization(value: f64, min: f64, max: f64) -> f64 {
    (value - min) / (max - min)
}

fn field_height(game: &Game, x: usize) -> u16 {
    let height = game
        .field()
        .block_rows()
        .enumerate()
        .find(|(_y, row)| !row[x].is_empty())
        .map_or(0, |(y, _row)| Field::BLOCKS_HEIGHT - y);

    u16::try_from(height).unwrap()
}

fn field_height_max(game: &Game) -> u16 {
    let max = (0..Field::BLOCKS_HEIGHT)
        .find(|&y| {
            game.field()
                .block_row(y)
                .iter()
                .any(|block| !block.is_empty())
        })
        .map_or(0, |y| Field::BLOCKS_HEIGHT - y);

    u16::try_from(max).unwrap()
}

fn diff_in_height(game: &Game) -> u16 {
    let mut diff = 0;
    let mut top = [0; Field::BLOCKS_WIDTH];

    for (x, top) in top.iter_mut().enumerate() {
        *top = field_height(game, x);
    }
    for i in 0..top.len() - 1 {
        diff += top[i].abs_diff(top[i + 1]);
    }

    diff
}

fn dead_space_count(game: &Game) -> u16 {
    let count = (0..Field::BLOCKS_WIDTH)
        .map(|x| {
            game.field()
                .block_rows()
                .map(|row| row[x])
                .skip_while(|block| block.is_empty())
                .filter(|block| block.is_empty())
                .count()
        })
        .sum::<usize>();

    u16::try_from(count).unwrap()
}
