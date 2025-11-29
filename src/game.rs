use std::collections::VecDeque;

use crate::{
    field::{Field, Position},
    mino::{self, MinoKind, MinoShape},
};

const SCORE_TABLE: [usize; 5] = [0, 1, 5, 25, 100];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DropResult {
    Success { lines_cleared: usize },
    GameOver,
}

impl DropResult {
    pub(crate) fn is_gameover(self) -> bool {
        matches!(self, DropResult::GameOver)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GameState {
    Playing,
    Paused,
    GameOver,
}

impl GameState {
    pub(crate) fn is_paused(&self) -> bool {
        matches!(self, GameState::Paused)
    }

    pub(crate) fn is_gameover(&self) -> bool {
        matches!(self, GameState::GameOver)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Game {
    field: Field,
    falling_mino_position: Position,
    falling_mino: MinoShape,
    held_mino: Option<MinoShape>,
    hold_used: bool,
    next_minos: VecDeque<MinoShape>,
    next_minos_queue: VecDeque<MinoShape>,
    score: usize,
    cleared_lines: usize,
    state: GameState,
}

impl Game {
    pub(crate) fn new() -> Self {
        let mut game = Self {
            field: Field::INITIAL,
            falling_mino_position: Position::INITIAL,
            falling_mino: *rand::random::<MinoKind>().shape(),
            held_mino: None,
            hold_used: false,
            next_minos: mino::gen_mino_7().into(),
            next_minos_queue: mino::gen_mino_7().into(),
            score: 0,
            cleared_lines: 0,
            state: GameState::Playing,
        };
        game.begin_next_mino_fall();
        game
    }

    pub(crate) fn level(&self) -> usize {
        self.cleared_lines / 10
    }

    pub(crate) fn cleared_lines(&self) -> usize {
        self.cleared_lines
    }

    pub(crate) fn score(&self) -> usize {
        self.score
    }

    pub(crate) fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            GameState::GameOver => GameState::GameOver, // No change from game over
        };
    }

    pub(crate) fn state(&self) -> &GameState {
        &self.state
    }

    pub(crate) fn field(&self) -> &Field {
        &self.field
    }

    pub(crate) fn falling_mino(&self) -> (Position, &MinoShape) {
        (self.falling_mino_position, &self.falling_mino)
    }

    pub(crate) fn held_mino(&self) -> &Option<MinoShape> {
        &self.held_mino
    }

    pub(crate) fn next_minos(&self) -> &VecDeque<MinoShape> {
        &self.next_minos
    }

    pub(crate) fn simulate_drop_position(&self) -> Position {
        let mut drop_pos = self.falling_mino_position;
        loop {
            let Some(new_pos) = drop_pos.down() else {
                break;
            };
            if self.field.is_colliding(&new_pos, &self.falling_mino) {
                break;
            }
            drop_pos = new_pos;
        }
        drop_pos
    }

    pub(crate) fn try_move_left(&mut self) -> Result<(), ()> {
        let new_pos = self.falling_mino_position.left().ok_or(())?;
        self.try_move(new_pos)
    }

    pub(crate) fn try_move_right(&mut self) -> Result<(), ()> {
        let new_pos = self.falling_mino_position.right().ok_or(())?;
        self.try_move(new_pos)
    }

    fn try_move(&mut self, new_pos: Position) -> Result<(), ()> {
        if self.field.is_colliding(&new_pos, &self.falling_mino) {
            return Err(());
        }
        self.falling_mino_position = new_pos;
        Ok(())
    }

    pub(crate) fn try_rotate_left(&mut self) -> Result<(), ()> {
        let mut new_shape = MinoShape::default();
        for (y, row) in new_shape.iter_mut().enumerate() {
            for (x, v) in row.iter_mut().enumerate() {
                *v = self.falling_mino[x][4 - 1 - y];
            }
        }
        if self
            .field
            .is_colliding(&self.falling_mino_position, &new_shape)
        {
            let new_pos = super_rotation(&self.field, &self.falling_mino_position, &new_shape)?;
            self.falling_mino_position = new_pos;
        }
        self.falling_mino = new_shape;
        Ok(())
    }

    pub(crate) fn try_rotate_right(&mut self) -> Result<(), ()> {
        let mut new_shape = MinoShape::default();
        for (y, row) in new_shape.iter_mut().enumerate() {
            for (x, v) in row.iter_mut().enumerate() {
                *v = self.falling_mino[4 - 1 - x][y];
            }
        }
        if self
            .field
            .is_colliding(&self.falling_mino_position, &new_shape)
        {
            let new_pos = super_rotation(&self.field, &self.falling_mino_position, &new_shape)?;
            self.falling_mino_position = new_pos;
        }
        self.falling_mino = new_shape;
        Ok(())
    }

    pub(crate) fn try_hold(&mut self) -> Result<(), ()> {
        if self.hold_used {
            return Err(());
        }
        if let Some(mut hold) = self.held_mino {
            let new_pos = Position::INITIAL;
            if self.field.is_colliding(&new_pos, &hold) {
                return Err(());
            }
            std::mem::swap(&mut hold, &mut self.falling_mino);
            self.held_mino = Some(hold);
            self.falling_mino_position = new_pos;
        } else {
            self.held_mino = Some(self.falling_mino);
            self.begin_next_mino_fall();
        }
        self.hold_used = true;
        Ok(())
    }

    pub(crate) fn try_soft_drop(&mut self) -> Result<(), ()> {
        let new_pos = self.falling_mino_position.down().ok_or(())?;
        self.try_move(new_pos)?;
        Ok(())
    }

    pub(crate) fn hard_drop_and_complete(&mut self) -> DropResult {
        while self.try_soft_drop().is_ok() {}
        self.complete_mino_drop()
    }

    pub(crate) fn auto_drop_and_complete(&mut self) -> DropResult {
        if self.try_soft_drop().is_ok() {
            return DropResult::Success { lines_cleared: 0 };
        }
        self.complete_mino_drop()
    }

    fn complete_mino_drop(&mut self) -> DropResult {
        self.field
            .fill_mino(&self.falling_mino_position, &self.falling_mino);
        let line = self.field.clear_lines();
        self.score += SCORE_TABLE[line];
        self.cleared_lines += line;

        self.begin_next_mino_fall();
        if self
            .field
            .is_colliding(&self.falling_mino_position, &self.falling_mino)
        {
            self.state = GameState::GameOver;
            return DropResult::GameOver;
        }

        self.hold_used = false;
        DropResult::Success {
            lines_cleared: line,
        }
    }

    fn begin_next_mino_fall(&mut self) {
        self.falling_mino_position = Position::INITIAL;
        self.falling_mino = self.next_minos.pop_front().unwrap();
        if let Some(next) = self.next_minos_queue.pop_front() {
            self.next_minos.push_back(next);
        } else {
            self.next_minos_queue = mino::gen_mino_7().into();
            self.next_minos
                .push_back(self.next_minos_queue.pop_front().unwrap());
        }
    }
}

fn super_rotation(field: &Field, pos: &Position, mino: &MinoShape) -> Result<Position, ()> {
    let diff_pos = [pos.up(), pos.right(), pos.down(), pos.left()];
    for pos in diff_pos.iter().flatten() {
        if !field.is_colliding(pos, mino) {
            return Ok(*pos);
        }
    }
    Err(())
}
