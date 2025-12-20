use std::time::Duration;

use crate::{
    HoldError, PieceCollisionError,
    core::{
        piece::{Piece, PieceKind},
        render_board::RenderBoard,
    },
};

use super::state::GameState;

#[derive(Debug, Clone, PartialEq, Eq, derive_more::IsVariant)]
pub enum SessionState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    game_state: GameState,
    render_board: RenderBoard,
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
            game_state: GameState::new(),
            render_board: RenderBoard::INITIAL,
            session_state: SessionState::Playing,
            fps,
            total_frames: 0,
            drop_frames: drop_frames(0, fps),
        }
    }

    #[must_use]
    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    #[must_use]
    pub fn session_state(&self) -> &SessionState {
        &self.session_state
    }

    #[must_use]
    pub fn level(&self) -> usize {
        self.game_state.level()
    }

    #[must_use]
    pub fn total_cleared_lines(&self) -> usize {
        self.game_state.total_cleared_lines()
    }

    #[must_use]
    pub fn completed_pieces(&self) -> usize {
        self.game_state.completed_pieces()
    }

    #[must_use]
    pub fn line_cleared_counter(&self) -> &[usize; 5] {
        self.game_state.line_cleared_counter()
    }

    #[must_use]
    pub fn score(&self) -> usize {
        self.game_state.score()
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
    pub fn render_board(&self) -> &RenderBoard {
        &self.render_board
    }

    #[must_use]
    pub fn falling_piece(&self) -> &Piece {
        self.game_state.falling_piece()
    }

    #[must_use]
    pub fn held_piece(&self) -> Option<PieceKind> {
        self.game_state.held_piece()
    }

    pub fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.game_state.next_pieces()
    }

    #[must_use]
    pub fn simulate_drop_position(&self) -> Piece {
        self.game_state.simulate_drop_position()
    }

    pub fn increment_frame(&mut self) {
        self.total_frames += 1;
        self.drop_frames = self.drop_frames.saturating_sub(1);
        if self.drop_frames == 0 {
            self.drop_frames = drop_frames(self.level() as u64, self.fps);
            self.auto_drop_and_complete();
        }
    }

    pub fn try_move_left(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .game_state
            .falling_piece()
            .left()
            .ok_or(PieceCollisionError)?;
        self.game_state.set_falling_piece(piece)
    }

    pub fn try_move_right(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .game_state
            .falling_piece()
            .right()
            .ok_or(PieceCollisionError)?;
        self.game_state.set_falling_piece(piece)
    }

    pub fn try_soft_drop(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .game_state
            .falling_piece()
            .down()
            .ok_or(PieceCollisionError)?;
        self.game_state.set_falling_piece(piece)
    }

    pub fn try_rotate_left(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .game_state
            .falling_piece()
            .super_rotated_left(self.game_state.board())
            .ok_or(PieceCollisionError)?;
        self.game_state.set_falling_piece_unchecked(piece);
        Ok(())
    }

    pub fn try_rotate_right(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .game_state
            .falling_piece()
            .super_rotated_right(self.game_state.board())
            .ok_or(PieceCollisionError)?;
        self.game_state.set_falling_piece_unchecked(piece);
        Ok(())
    }

    pub fn try_hold(&mut self) -> Result<(), HoldError> {
        self.game_state.try_hold()
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
        self.render_board
            .fill_piece(self.game_state.falling_piece());
        let Ok(cleared_lines) = self.game_state.complete_piece_drop() else {
            self.session_state = SessionState::GameOver;
            return;
        };
        assert_eq!(self.render_board.clear_lines(), cleared_lines);
    }
}
