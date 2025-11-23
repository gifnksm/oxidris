use std::iter;

use crate::{
    ga::{GenoSeq, GenomeKind},
    game::{self, Game},
};

pub(crate) fn eval(game: &Game, weight: &GenoSeq) -> (Game, bool) {
    let mut elite = ((game.clone(), true), f64::MIN);

    for mut game in next_candidates(game) {
        let _ = game::try_hard_drop(&mut game);
        let Ok(pre_line) = game::landing(&mut game) else {
            let score = compute_score(&game, weight, true, 0);
            if elite.1 < score {
                elite.0 = (game, true);
                elite.1 = score;
            }
            continue;
        };

        for mut game in next_candidates(&game) {
            let _ = game::try_hard_drop(&mut game);
            let (line, gameover) = match game::landing(&mut game) {
                Ok(line) => (line, false),
                Err(()) => (0, true),
            };
            let score = compute_score(&game, weight, gameover, pre_line + line);
            if elite.1 < score {
                elite.0 = (game, gameover);
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

fn compute_score(game: &Game, weight: &GenoSeq, gameover: bool, line: usize) -> f64 {
    if gameover {
        return 0.0;
    }

    let height_max = field_height_max(game);
    let height_diff = diff_in_height(game);
    let dead_space = dead_space_count(game);

    let mut line = normalization(line as f64, 0.0, 8.0);
    let mut height_max = 1.0 - normalization(height_max as f64, 0.0, 20.0);
    let mut height_diff = 1.0 - normalization(height_diff as f64, 0.0, 200.0);
    let mut dead_space = 1.0 - normalization(dead_space as f64, 0.0, 200.0);

    line *= weight[GenomeKind::Line] as f64;
    height_max *= weight[GenomeKind::HeightMax] as f64;
    height_diff *= weight[GenomeKind::HeightDiff] as f64;
    dead_space *= weight[GenomeKind::DeadSpace] as f64;

    line + height_max + height_diff + dead_space
}

fn hold(game: &Game) -> Option<Game> {
    let mut game = game.clone();
    game::try_hold(&mut game).ok()?;
    Some(game)
}

fn rotate_right(game: &Game) -> Option<Game> {
    let mut game = game.clone();
    game::try_rotate_right(&mut game).ok()?;
    Some(game)
}

fn move_left(game: &Game) -> Option<Game> {
    let mut game = game.clone();
    if game::try_move_left(&mut game).is_err() {
        game::try_drop(&mut game).ok()?;
        game::try_move_left(&mut game).ok()?;
    }
    Some(game)
}

fn move_right(game: &Game) -> Option<Game> {
    let mut game = game.clone();
    if game::try_move_right(&mut game).is_err() {
        game::try_drop(&mut game).ok()?;
        game::try_move_right(&mut game).ok()?;
    }
    Some(game)
}

fn normalization(value: f64, min: f64, max: f64) -> f64 {
    (value - min) / (max - min)
}

fn field_height(game: &Game, x: usize) -> usize {
    game.block_rows()
        .enumerate()
        .find(|(_y, row)| !row[x].is_empty())
        .map(|(y, _row)| Game::BLOCKS_HEIGHT - y)
        .unwrap_or(0)
}

fn field_height_max(game: &Game) -> usize {
    (0..Game::BLOCKS_HEIGHT)
        .find(|&y| game.block_row(y).iter().any(|block| !block.is_empty()))
        .map(|y| Game::BLOCKS_HEIGHT - y)
        .unwrap_or(0)
}

fn diff_in_height(game: &Game) -> usize {
    let mut diff = 0;
    let mut top = [0; Game::BLOCKS_WIDTH];

    for (x, top) in top.iter_mut().enumerate() {
        *top = field_height(game, x);
    }
    for i in 0..top.len() - 1 {
        diff += top[i].abs_diff(top[i + 1]);
    }

    diff
}

fn dead_space_count(game: &Game) -> usize {
    (0..Game::BLOCKS_WIDTH)
        .map(|x| {
            game.block_rows()
                .map(|row| row[x])
                .skip_while(|block| block.is_empty())
                .filter(|block| block.is_empty())
                .count()
        })
        .sum()
}
