use std::time::Duration;

use crate::{
    field::Field,
    mino::{Mino, MinoKind},
};

use super::core::GameCore;

#[derive(Debug, Clone, PartialEq, Eq, derive_more::IsVariant)]
pub(crate) enum GameState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug, Clone)]
pub(crate) struct GameUi {
    core: GameCore,
    state: GameState,
    fps: u64,
    total_frames: u64,
    drop_frames: u64,
}

fn drop_frames(level: u64, fps: u64) -> u64 {
    let millis = 100 + u64::saturating_sub(900, level * 100);
    millis * fps / 1000
}

impl GameUi {
    pub(crate) fn new(fps: u64) -> Self {
        Self {
            core: GameCore::new(),
            state: GameState::Playing,
            fps,
            total_frames: 0,
            drop_frames: drop_frames(0, fps),
        }
    }

    pub(crate) fn core(&self) -> &GameCore {
        &self.core
    }

    pub(crate) fn level(&self) -> usize {
        self.core.level()
    }

    pub(crate) fn cleared_lines(&self) -> usize {
        self.core.cleared_lines()
    }

    pub(crate) fn score(&self) -> usize {
        self.core.score()
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
        self.core.field()
    }

    pub(crate) fn falling_mino(&self) -> &Mino {
        self.core.falling_mino()
    }

    pub(crate) fn held_mino(&self) -> Option<MinoKind> {
        self.core.held_mino()
    }

    pub(crate) fn next_minos(&self) -> impl Iterator<Item = MinoKind> + '_ {
        self.core.next_minos()
    }

    pub(crate) fn simulate_drop_position(&self) -> Mino {
        self.core.simulate_drop_position()
    }

    pub(crate) fn increment_frame(&mut self) {
        self.total_frames += 1;
        self.drop_frames = self.drop_frames.saturating_sub(1);
        if self.drop_frames == 0 {
            self.drop_frames = drop_frames(self.level() as u64, self.fps);
            self.auto_drop_and_complete();
        }
    }

    pub(crate) fn try_move_left(&mut self) -> Result<(), ()> {
        let mino = self.core.falling_mino().left().ok_or(())?;
        self.core.set_falling_mino(mino)
    }

    pub(crate) fn try_move_right(&mut self) -> Result<(), ()> {
        let mino = self.core.falling_mino().right().ok_or(())?;
        self.core.set_falling_mino(mino)
    }

    pub(crate) fn try_soft_drop(&mut self) -> Result<(), ()> {
        let mino = self.core.falling_mino().down().ok_or(())?;
        self.core.set_falling_mino(mino)
    }

    pub(crate) fn try_rotate_left(&mut self) -> Result<(), ()> {
        let mino = self
            .core
            .falling_mino()
            .super_rotated_left(self.core.field())
            .ok_or(())?;
        self.core.set_falling_mino_unchecked(mino);
        Ok(())
    }

    pub(crate) fn try_rotate_right(&mut self) -> Result<(), ()> {
        let mino = self
            .core
            .falling_mino()
            .super_rotated_right(self.core.field())
            .ok_or(())?;
        self.core.set_falling_mino_unchecked(mino);
        Ok(())
    }

    pub(crate) fn try_hold(&mut self) -> Result<(), ()> {
        self.core.try_hold()
    }

    pub(crate) fn hard_drop_and_complete(&mut self) {
        while self.try_soft_drop().is_ok() {}
        self.complete_mino_drop();
    }

    pub(crate) fn auto_drop_and_complete(&mut self) {
        if self.try_soft_drop().is_ok() {
            return;
        }
        self.complete_mino_drop();
    }

    fn complete_mino_drop(&mut self) {
        if self.core.complete_mino_drop().is_err() {
            self.state = GameState::GameOver;
        }
    }
}
