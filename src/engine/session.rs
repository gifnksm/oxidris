use std::time::Duration;

use crate::core::{
    piece::{Piece, PieceKind},
    render_board::RenderBoard,
};

use super::state::GameState;

#[derive(Debug, Clone, PartialEq, Eq, derive_more::IsVariant)]
pub(crate) enum SessionState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug, Clone)]
pub(crate) struct GameSession {
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
    pub(crate) fn new(fps: u64) -> Self {
        Self {
            game_state: GameState::new(),
            render_board: RenderBoard::INITIAL,
            session_state: SessionState::Playing,
            fps,
            total_frames: 0,
            drop_frames: drop_frames(0, fps),
        }
    }

    pub(crate) fn game_state(&self) -> &GameState {
        &self.game_state
    }

    pub(crate) fn session_state(&self) -> &SessionState {
        &self.session_state
    }

    pub(crate) fn level(&self) -> usize {
        self.game_state.level()
    }

    pub(crate) fn cleared_lines(&self) -> usize {
        self.game_state.cleared_lines()
    }

    pub(crate) fn score(&self) -> usize {
        self.game_state.score()
    }

    pub(crate) fn duration(&self) -> Duration {
        const NANOS_PER_SEC: u64 = 1_000_000_000;
        let secs = self.total_frames / self.fps;
        let nanos = (self.total_frames % self.fps) * NANOS_PER_SEC / self.fps;
        Duration::new(secs, nanos.try_into().unwrap())
    }

    pub(crate) fn toggle_pause(&mut self) {
        self.session_state = match self.session_state {
            SessionState::Playing => SessionState::Paused,
            SessionState::Paused => SessionState::Playing,
            SessionState::GameOver => SessionState::GameOver, // No change from game over
        };
    }

    pub(crate) fn render_board(&self) -> &RenderBoard {
        &self.render_board
    }

    pub(crate) fn falling_piece(&self) -> &Piece {
        self.game_state.falling_piece()
    }

    pub(crate) fn held_piece(&self) -> Option<PieceKind> {
        self.game_state.held_piece()
    }

    pub(crate) fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.game_state.next_pieces()
    }

    pub(crate) fn simulate_drop_position(&self) -> Piece {
        self.game_state.simulate_drop_position()
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
        let piece = self.game_state.falling_piece().left().ok_or(())?;
        self.game_state.set_falling_piece(piece)
    }

    pub(crate) fn try_move_right(&mut self) -> Result<(), ()> {
        let piece = self.game_state.falling_piece().right().ok_or(())?;
        self.game_state.set_falling_piece(piece)
    }

    pub(crate) fn try_soft_drop(&mut self) -> Result<(), ()> {
        let piece = self.game_state.falling_piece().down().ok_or(())?;
        self.game_state.set_falling_piece(piece)
    }

    pub(crate) fn try_rotate_left(&mut self) -> Result<(), ()> {
        let piece = self
            .game_state
            .falling_piece()
            .super_rotated_left(self.game_state.board())
            .ok_or(())?;
        self.game_state.set_falling_piece_unchecked(piece);
        Ok(())
    }

    pub(crate) fn try_rotate_right(&mut self) -> Result<(), ()> {
        let piece = self
            .game_state
            .falling_piece()
            .super_rotated_right(self.game_state.board())
            .ok_or(())?;
        self.game_state.set_falling_piece_unchecked(piece);
        Ok(())
    }

    pub(crate) fn try_hold(&mut self) -> Result<(), ()> {
        self.game_state.try_hold()
    }

    pub(crate) fn hard_drop_and_complete(&mut self) {
        while self.try_soft_drop().is_ok() {}
        self.complete_piece_drop();
    }

    pub(crate) fn auto_drop_and_complete(&mut self) {
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
