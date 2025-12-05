use std::time::Duration;

use crate::{
    field::Field,
    mino::{Mino, MinoGenerator, MinoKind},
};

const SCORE_TABLE: [usize; 5] = [0, 1, 5, 25, 100];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DropResult {
    Success { lines_cleared: usize },
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, derive_more::IsVariant)]
pub(crate) enum GameState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug, Clone)]
pub(crate) struct Game {
    field: Field,
    falling_mino: Mino,
    held_mino: Option<MinoKind>,
    hold_used: bool,
    mino_generator: MinoGenerator,
    score: usize,
    cleared_lines: usize,
    state: GameState,
    fps: u64,
    total_frames: u64,
    drop_frames: u64,
}

fn drop_frames(level: u64, fps: u64) -> u64 {
    let millis = 100 + u64::saturating_sub(900, level * 100);
    millis * fps / 1000
}

impl Game {
    pub(crate) fn new(fps: u64) -> Self {
        let first_mino = MinoKind::I; // dummy initial value
        let mut game = Self {
            field: Field::INITIAL,
            falling_mino: Mino::new(first_mino),
            held_mino: None,
            hold_used: false,
            mino_generator: MinoGenerator::new(),
            score: 0,
            cleared_lines: 0,
            state: GameState::Playing,
            fps,
            total_frames: 0,
            drop_frames: drop_frames(0, fps),
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

    pub(crate) fn duration(&self) -> Duration {
        const NANOS_PER_SEC: u64 = 1_000_000_000;
        let secs = self.total_frames / self.fps;
        let nanos = (self.total_frames % self.fps) * NANOS_PER_SEC / self.fps;
        Duration::new(secs, nanos.try_into().unwrap())
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

    pub(crate) fn falling_mino(&self) -> &Mino {
        &self.falling_mino
    }

    pub(crate) fn held_mino(&self) -> Option<MinoKind> {
        self.held_mino
    }

    pub(crate) fn next_minos(&self) -> impl Iterator<Item = MinoKind> + '_ {
        self.mino_generator.next_minos()
    }

    pub(crate) fn simulate_drop_position(&self) -> Mino {
        let mut dropped = self.falling_mino;
        while let Some(mino) = dropped.down() {
            if self.field.is_colliding(&mino) {
                break;
            }
            dropped = mino;
        }
        dropped
    }

    pub(crate) fn increment_frame(&mut self) -> Option<DropResult> {
        self.total_frames += 1;
        self.drop_frames = self.drop_frames.saturating_sub(1);
        if self.drop_frames == 0 {
            self.drop_frames = drop_frames(self.level() as u64, self.fps);
            Some(self.auto_drop_and_complete())
        } else {
            None
        }
    }

    fn try_update_falling_mino(&mut self, mino: Mino) -> Result<(), ()> {
        if self.field.is_colliding(&mino) {
            return Err(());
        }
        self.falling_mino = mino;
        Ok(())
    }

    pub(crate) fn try_move_left(&mut self) -> Result<(), ()> {
        let mino = self.falling_mino.left().ok_or(())?;
        self.try_update_falling_mino(mino)
    }

    pub(crate) fn try_move_right(&mut self) -> Result<(), ()> {
        let mino = self.falling_mino.right().ok_or(())?;
        self.try_update_falling_mino(mino)
    }

    pub(crate) fn try_rotate_left(&mut self) -> Result<(), ()> {
        let mut mino = self.falling_mino.rotated_left();
        if self.field.is_colliding(&mino) {
            mino = super_rotation(&self.field, &mino)?;
        }
        self.falling_mino = mino;
        Ok(())
    }

    pub(crate) fn try_rotate_right(&mut self) -> Result<(), ()> {
        let mut mino = self.falling_mino.rotated_right();
        if self.field.is_colliding(&mino) {
            mino = super_rotation(&self.field, &mino)?;
        }
        self.falling_mino = mino;
        Ok(())
    }

    pub(crate) fn try_hold(&mut self) -> Result<(), ()> {
        if self.hold_used {
            return Err(());
        }
        if let Some(held_mino) = self.held_mino {
            let mino = Mino::new(held_mino);
            if self.field.is_colliding(&mino) {
                return Err(());
            }
            self.held_mino = Some(self.falling_mino.kind());
            self.falling_mino = mino;
        } else {
            self.held_mino = Some(self.falling_mino.kind());
            self.begin_next_mino_fall();
        }
        self.hold_used = true;
        Ok(())
    }

    pub(crate) fn try_soft_drop(&mut self) -> Result<(), ()> {
        let mino = self.falling_mino.down().ok_or(())?;
        self.try_update_falling_mino(mino)
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
        self.field.fill_mino(&self.falling_mino);
        let line = self.field.clear_lines();
        self.score += SCORE_TABLE[line];
        self.cleared_lines += line;

        self.begin_next_mino_fall();
        if self.field.is_colliding(&self.falling_mino) {
            self.state = GameState::GameOver;
            return DropResult::GameOver;
        }

        self.hold_used = false;
        DropResult::Success {
            lines_cleared: line,
        }
    }

    fn begin_next_mino_fall(&mut self) {
        self.falling_mino = Mino::new(self.mino_generator.pop_next());
    }
}

fn super_rotation(field: &Field, mino: &Mino) -> Result<Mino, ()> {
    let minos = [mino.up(), mino.right(), mino.down(), mino.left()];
    for mino in minos.iter().flatten() {
        if !field.is_colliding(mino) {
            return Ok(*mino);
        }
    }
    Err(())
}
