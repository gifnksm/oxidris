use std::collections::VecDeque;

use crate::{
    block::BlockKind,
    mino::{self, MinoKind, MinoShape},
};

const FIELD_WIDTH: usize = 12 + 2;
const FIELD_HEIGHT: usize = 22 + 1;

pub(crate) type FieldSize = [[BlockKind; FIELD_WIDTH]; FIELD_HEIGHT];

#[derive(Debug, Clone, Copy)]
pub(crate) struct Position {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl Position {
    const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    const fn init() -> Self {
        Self::new(5, 0)
    }

    const fn left(&self) -> Option<Self> {
        if self.x == 0 {
            None
        } else {
            Some(Self::new(self.x - 1, self.y))
        }
    }

    const fn right(&self) -> Option<Self> {
        if self.x >= FIELD_WIDTH - 1 {
            None
        } else {
            Some(Self::new(self.x + 1, self.y))
        }
    }

    const fn up(&self) -> Option<Self> {
        if self.y == 0 {
            None
        } else {
            Some(Self::new(self.x, self.y - 1))
        }
    }

    const fn down(&self) -> Option<Self> {
        if self.y >= FIELD_HEIGHT - 1 {
            None
        } else {
            Some(Self::new(self.x, self.y + 1))
        }
    }
}

const SCORE_TABLE: [usize; 5] = [0, 1, 5, 25, 100];

#[derive(Debug, Clone)]
pub(crate) struct Game {
    field: FieldSize,
    pos: Position,
    mino: MinoShape,
    hold: Option<MinoShape>,
    holded: bool,
    next: VecDeque<MinoShape>,
    next_buf: VecDeque<MinoShape>,
    score: usize,
    line: usize,
}

impl Game {
    pub(crate) fn new() -> Self {
        use BlockKind::{Empty as E, Wall as W};
        let mut game = Self {
            field: [
                [E, W, W, W, E, E, E, E, E, E, W, W, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, E, E, E, E, E, E, E, E, E, E, W, E],
                [E, W, W, W, W, W, W, W, W, W, W, W, W, E],
                [E, E, E, E, E, E, E, E, E, E, E, E, E, E],
            ],
            pos: Position::init(),
            mino: *rand::random::<MinoKind>().shape(),
            hold: None,
            holded: false,
            next: mino::gen_mino_7().into(),
            next_buf: mino::gen_mino_7().into(),
            score: 0,
            line: 0,
        };
        spawn_mino(&mut game).unwrap();
        game
    }

    pub(crate) const BLOCKS_WIDTH: usize = FIELD_WIDTH - 4;
    pub(crate) const BLOCKS_HEIGHT: usize = FIELD_HEIGHT - 3;

    pub(crate) fn block_row(&self, y: usize) -> &[BlockKind] {
        &self.field[y + 1][2..FIELD_WIDTH - 2]
    }

    pub(crate) fn block_rows(&self) -> impl Iterator<Item = &[BlockKind]> {
        self.field[1..FIELD_HEIGHT - 2]
            .iter()
            .map(|row| &row[2..FIELD_WIDTH - 2])
    }

    pub(crate) fn level(&self) -> usize {
        self.line / 10
    }

    pub(crate) fn line(&self) -> usize {
        self.line
    }

    pub(crate) fn score(&self) -> usize {
        self.score
    }

    pub(crate) fn field(&self) -> &FieldSize {
        &self.field
    }

    pub(crate) fn pos(&self) -> &Position {
        &self.pos
    }

    pub(crate) fn mino(&self) -> &MinoShape {
        &self.mino
    }

    pub(crate) fn hold(&self) -> &Option<MinoShape> {
        &self.hold
    }

    pub(crate) fn next(&self) -> &VecDeque<MinoShape> {
        &self.next
    }
}

pub(crate) fn ghost_pos(field: &FieldSize, pos: &Position, mino: &MinoShape) -> Position {
    let mut ghost_pos = *pos;
    loop {
        let Some(new_pos) = ghost_pos.down() else {
            break;
        };
        if is_collision(field, &new_pos, mino) {
            break;
        }
        ghost_pos = new_pos;
    }
    ghost_pos
}

pub(crate) fn is_collision(field: &FieldSize, pos: &Position, mino: &MinoShape) -> bool {
    for y in 0..4 {
        for x in 0..4 {
            if y + pos.y >= FIELD_HEIGHT || x + pos.x >= FIELD_WIDTH {
                continue;
            }
            let block = mino[y][x];
            if !block.is_empty() && !field[y + pos.y][x + pos.x].is_empty() {
                return true;
            }
        }
    }
    false
}

fn try_move(game: &mut Game, new_pos: Position) -> Result<(), ()> {
    if is_collision(&game.field, &new_pos, &game.mino) {
        return Err(());
    }
    game.pos = new_pos;
    Ok(())
}

pub(crate) fn try_move_left(game: &mut Game) -> Result<(), ()> {
    let new_pos = game.pos.left().ok_or(())?;
    try_move(game, new_pos)?;
    Ok(())
}

pub(crate) fn try_move_right(game: &mut Game) -> Result<(), ()> {
    let new_pos = game.pos.right().ok_or(())?;
    try_move(game, new_pos)?;
    Ok(())
}

fn super_rotation(field: &FieldSize, pos: &Position, mino: &MinoShape) -> Result<Position, ()> {
    let diff_pos = [pos.up(), pos.right(), pos.down(), pos.left()];
    for pos in diff_pos.iter().flatten() {
        if !is_collision(field, pos, mino) {
            return Ok(*pos);
        }
    }
    Err(())
}

pub(crate) fn try_rotate_left(game: &mut Game) -> Result<(), ()> {
    let mut new_shape = MinoShape::default();
    for (y, row) in new_shape.iter_mut().enumerate() {
        for (x, v) in row.iter_mut().enumerate() {
            *v = game.mino[x][4 - 1 - y];
        }
    }
    if is_collision(&game.field, &game.pos, &new_shape) {
        let new_pos = super_rotation(&game.field, &game.pos, &new_shape)?;
        game.pos = new_pos;
    }
    game.mino = new_shape;
    Ok(())
}

pub(crate) fn try_rotate_right(game: &mut Game) -> Result<(), ()> {
    let mut new_shape = MinoShape::default();
    for (y, row) in new_shape.iter_mut().enumerate() {
        for (x, v) in row.iter_mut().enumerate() {
            *v = game.mino[4 - 1 - x][y];
        }
    }
    if is_collision(&game.field, &game.pos, &new_shape) {
        let new_pos = super_rotation(&game.field, &game.pos, &new_shape)?;
        game.pos = new_pos;
    }
    game.mino = new_shape;
    Ok(())
}

pub(crate) fn try_drop(game: &mut Game) -> Result<(), ()> {
    let new_pos = game.pos.down().ok_or(())?;
    try_move(game, new_pos)?;
    Ok(())
}

pub(crate) fn try_hard_drop(game: &mut Game) -> Result<(), ()> {
    try_drop(game)?;
    while try_drop(game).is_ok() {}
    Ok(())
}

pub(crate) fn try_hold(game: &mut Game) -> Result<(), ()> {
    if game.holded {
        return Err(());
    }
    if let Some(mut hold) = game.hold {
        let new_pos = Position::init();
        if is_collision(&game.field, &new_pos, &hold) {
            return Err(());
        }
        std::mem::swap(&mut hold, &mut game.mino);
        game.hold = Some(hold);
        game.pos = new_pos;
    } else {
        game.hold = Some(game.mino);
        spawn_mino(game).ok();
    }
    game.holded = true;
    Ok(())
}

pub(crate) fn landing(game: &mut Game) -> Result<usize, ()> {
    fix_mino(game);
    let line = erase_line(&mut game.field);
    game.score += SCORE_TABLE[line];
    game.line += line;
    spawn_mino(game)?;
    game.holded = false;
    Ok(line)
}

fn fix_mino(
    Game {
        field, pos, mino, ..
    }: &mut Game,
) {
    for y in 0..4 {
        for x in 0..4 {
            let block = mino[y][x];
            if !block.is_empty() {
                field[y + pos.y][x + pos.x] = block;
            }
        }
    }
}

fn erase_line(field: &mut FieldSize) -> usize {
    let mut count = 0;
    for y in 1..FIELD_HEIGHT - 2 {
        let can_erase = field[y][1..FIELD_WIDTH - 1].iter().all(|&v| !v.is_empty());
        if can_erase {
            count += 1;
            for y2 in (2..=y).rev() {
                field[y2] = field[y2 - 1];
            }
        }
    }
    count
}

fn spawn_mino(game: &mut Game) -> Result<(), ()> {
    game.pos = Position::init();
    game.mino = game.next.pop_front().unwrap();
    if let Some(next) = game.next_buf.pop_front() {
        game.next.push_back(next);
    } else {
        game.next_buf = mino::gen_mino_7().into();
        game.next.push_back(game.next_buf.pop_front().unwrap());
    }
    if is_collision(&game.field, &game.pos, &game.mino) {
        return Err(());
    }
    Ok(())
}
