use std::time::Duration;

use crate::{
    HoldError, PieceCollisionError,
    core::{
        block_board::BlockBoard,
        piece::{Piece, PieceKind},
    },
};

use super::{GameStats, game_field::GameField};

#[derive(Debug, Clone, PartialEq, Eq, derive_more::IsVariant)]
pub enum SessionState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    field: GameField,
    stats: GameStats,
    render_board: BlockBoard,
    session_state: SessionState,
    fps: u64,
    total_frames: u64,
    drop_frames: u64,
}

fn drop_frames(level: u64, fps: u64) -> u64 {
    let millis = 100 + u64::saturating_sub(900, level * 100);
    millis * fps / 1000
}

impl GameSession {
    #[must_use]
    pub fn new(fps: u64) -> Self {
        Self {
            field: GameField::new(),
            stats: GameStats::new(),
            render_board: BlockBoard::INITIAL,
            session_state: SessionState::Playing,
            fps,
            total_frames: 0,
            drop_frames: drop_frames(0, fps),
        }
    }

    #[must_use]
    pub fn field(&self) -> &GameField {
        &self.field
    }

    #[must_use]
    pub fn stats(&self) -> &GameStats {
        &self.stats
    }

    #[must_use]
    pub fn session_state(&self) -> &SessionState {
        &self.session_state
    }

    #[must_use]
    pub fn duration(&self) -> Duration {
        const NANOS_PER_SEC: u64 = 1_000_000_000;
        let secs = self.total_frames / self.fps;
        let nanos = (self.total_frames % self.fps) * NANOS_PER_SEC / self.fps;
        Duration::new(secs, nanos.try_into().unwrap())
    }

    pub fn toggle_pause(&mut self) {
        self.session_state = match self.session_state {
            SessionState::Playing => SessionState::Paused,
            SessionState::Paused => SessionState::Playing,
            SessionState::GameOver => SessionState::GameOver, // No change from game over
        };
    }

    #[must_use]
    pub fn render_board(&self) -> &BlockBoard {
        &self.render_board
    }

    #[must_use]
    pub fn falling_piece(&self) -> &Piece {
        self.field.falling_piece()
    }

    #[must_use]
    pub fn held_piece(&self) -> Option<PieceKind> {
        self.field.held_piece()
    }

    pub fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.field.next_pieces()
    }

    #[must_use]
    pub fn simulate_drop_position(&self) -> Piece {
        self.field.simulate_drop_position()
    }

    pub fn increment_frame(&mut self) {
        self.total_frames += 1;
        self.drop_frames = self.drop_frames.saturating_sub(1);
        if self.drop_frames == 0 {
            self.drop_frames = drop_frames(self.stats().level() as u64, self.fps);
            self.auto_drop_and_complete();
        }
    }

    pub fn try_move_left(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .left()
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece(piece)
    }

    pub fn try_move_right(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .right()
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece(piece)
    }

    pub fn try_soft_drop(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .down()
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece(piece)
    }

    pub fn try_rotate_left(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .super_rotated_left(self.field.board())
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece_unchecked(piece);
        Ok(())
    }

    pub fn try_rotate_right(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .super_rotated_right(self.field.board())
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece_unchecked(piece);
        Ok(())
    }

    pub fn try_hold(&mut self) -> Result<(), HoldError> {
        self.field.try_hold()
    }

    pub fn hard_drop_and_complete(&mut self) {
        while self.try_soft_drop().is_ok() {}
        self.complete_piece_drop();
    }

    pub fn auto_drop_and_complete(&mut self) {
        if self.try_soft_drop().is_ok() {
            return;
        }
        self.complete_piece_drop();
    }

    fn complete_piece_drop(&mut self) {
        self.render_board.fill_piece(self.field.falling_piece());
        let (cleared_lines, result) = self.field.complete_piece_drop();
        self.stats.complete_piece_drop(cleared_lines);
        if result.is_err() {
            self.session_state = SessionState::GameOver;
            return;
        }
        assert_eq!(self.render_board.clear_lines(), cleared_lines);
    }
}
