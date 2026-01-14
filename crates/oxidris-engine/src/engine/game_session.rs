use std::time::Duration;

use crate::{
    HoldError, PieceCollisionError,
    core::{
        block_board::BlockBoard,
        piece::{Piece, PieceKind},
    },
};

use super::{GameStats, game_field::GameField};

/// Current state of a game session.
///
/// Tracks whether the game is actively running, paused, or has ended.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::IsVariant)]
pub enum SessionState {
    /// Game is actively running.
    Playing,
    /// Game is paused (time and input frozen).
    Paused,
    /// Game has ended (top-out condition reached).
    GameOver,
}

/// Multi-turn game session with statistics tracking and automatic gravity.
///
/// `GameSession` wraps [`GameField`] to provide a complete gameplay experience:
///
/// - **Automatic gravity**: Pieces drop automatically based on level
/// - **Statistics tracking**: Lines cleared, score, level progression
/// - **Hold restrictions**: Hold can only be used once per piece
/// - **Session state**: Playing, paused, or game over
/// - **Frame-based timing**: Integrates with game loop at specified FPS
///
/// This is the high-level API for human gameplay. For AI or low-level manipulation,
/// use [`GameField`] directly.
///
/// # Example
///
/// ```
/// use oxidris_engine::GameSession;
///
/// let mut session = GameSession::new(60); // 60 FPS
///
/// // Handle input
/// session.try_move_left().ok();
/// session.try_rotate_right().ok();
/// session.hard_drop_and_complete();
///
/// // Check game state
/// if session.session_state().is_game_over() {
///     println!("Game over! Score: {}", session.stats().score());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct GameSession {
    field: GameField,
    stats: GameStats,
    hold_used: bool,
    block_board: BlockBoard,
    session_state: SessionState,
    fps: u64,
    total_frames: u64,
    drop_frames: u64,
}

/// Calculates the number of frames between automatic drops based on level.
///
/// Drop speed increases with level:
///
/// - Level 0: 1000ms per drop (slowest)
/// - Level 1: 900ms per drop
/// - ...
/// - Level 9+: 100ms per drop (fastest)
fn drop_frames(level: u64, fps: u64) -> u64 {
    let millis = 100 + u64::saturating_sub(900, level * 100);
    millis * fps / 1000
}

impl GameSession {
    /// Creates a new game session with the specified frame rate.
    ///
    /// # Arguments
    ///
    /// * `fps` - Frames per second for timing calculations (typically 60)
    #[must_use]
    pub fn new(fps: u64) -> Self {
        Self {
            field: GameField::new(),
            stats: GameStats::new(),
            hold_used: false,
            block_board: BlockBoard::INITIAL,
            session_state: SessionState::Playing,
            fps,
            total_frames: 0,
            drop_frames: drop_frames(0, fps),
        }
    }

    /// Returns a reference to the underlying game field.
    #[must_use]
    pub fn field(&self) -> &GameField {
        &self.field
    }

    /// Returns a reference to the current game statistics.
    #[must_use]
    pub fn stats(&self) -> &GameStats {
        &self.stats
    }

    /// Returns whether hold has been used for the current piece.
    #[must_use]
    pub fn hold_used(&self) -> bool {
        self.hold_used
    }

    /// Returns the current session state (playing, paused, or game over).
    #[must_use]
    pub fn session_state(&self) -> &SessionState {
        &self.session_state
    }

    /// Returns the total elapsed time of the session based on frame count.
    #[must_use]
    pub fn duration(&self) -> Duration {
        const NANOS_PER_SEC: u64 = 1_000_000_000;
        let secs = self.total_frames / self.fps;
        let nanos = (self.total_frames % self.fps) * NANOS_PER_SEC / self.fps;
        Duration::new(secs, nanos.try_into().unwrap())
    }

    /// Toggles between playing and paused states.
    ///
    /// Has no effect if the game is already over.
    pub fn toggle_pause(&mut self) {
        self.session_state = match self.session_state {
            SessionState::Playing => SessionState::Paused,
            SessionState::Paused => SessionState::Playing,
            SessionState::GameOver => SessionState::GameOver, // No change from game over
        };
    }

    /// Returns the board with pieces rendered for display.
    #[must_use]
    pub fn block_board(&self) -> &BlockBoard {
        &self.block_board
    }

    /// Returns the currently falling piece.
    #[must_use]
    pub fn falling_piece(&self) -> Piece {
        self.field.falling_piece()
    }

    /// Returns the currently held piece, if any.
    #[must_use]
    pub fn held_piece(&self) -> Option<PieceKind> {
        self.field.held_piece()
    }

    /// Returns an iterator over the upcoming pieces in the queue.
    pub fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.field.next_pieces()
    }

    /// Simulates a hard drop and returns the final position without modifying state.
    #[must_use]
    pub fn simulate_drop_position(&self) -> Piece {
        self.field.simulate_drop_position()
    }

    /// Advances the game by one frame, applying automatic gravity if needed.
    ///
    /// Should be called once per frame in the game loop. Handles automatic piece
    /// dropping based on the current level and elapsed frames.
    pub fn increment_frame(&mut self) {
        self.total_frames += 1;
        self.drop_frames = self.drop_frames.saturating_sub(1);
        if self.drop_frames == 0 {
            self.drop_frames = drop_frames(self.stats().level() as u64, self.fps);
            self.auto_drop_and_complete();
        }
    }

    /// Attempts to move the falling piece one cell to the left.
    ///
    /// # Errors
    ///
    /// Returns `PieceCollisionError` if the move would cause a collision.
    pub fn try_move_left(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .left()
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece(piece)
    }

    /// Attempts to move the falling piece one cell to the right.
    ///
    /// # Errors
    ///
    /// Returns `PieceCollisionError` if the move would cause a collision.
    pub fn try_move_right(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .right()
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece(piece)
    }

    /// Attempts to move the falling piece one cell down (soft drop).
    ///
    /// # Errors
    ///
    /// Returns `PieceCollisionError` if the piece cannot move down further.
    pub fn try_soft_drop(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .down()
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece(piece)
    }

    /// Attempts to rotate the falling piece counterclockwise.
    ///
    /// Uses simplified wall kick system (see [Engine Implementation](../../../docs/architecture/engine/README.md)).
    ///
    /// # Errors
    ///
    /// Returns `PieceCollisionError` if rotation and all kicks fail.
    pub fn try_rotate_left(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .super_rotated_left(self.field.board())
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece_unchecked(piece);
        Ok(())
    }

    /// Attempts to rotate the falling piece clockwise.
    ///
    /// Uses simplified wall kick system (see [Engine Implementation](../../../docs/architecture/engine/README.md)).
    ///
    /// # Errors
    ///
    /// Returns `PieceCollisionError` if rotation and all kicks fail.
    pub fn try_rotate_right(&mut self) -> Result<(), PieceCollisionError> {
        let piece = self
            .field
            .falling_piece()
            .super_rotated_right(self.field.board())
            .ok_or(PieceCollisionError)?;
        self.field.set_falling_piece_unchecked(piece);
        Ok(())
    }

    /// Attempts to hold the current piece.
    ///
    /// Hold can only be used once per piece. The flag resets after the piece is locked.
    ///
    /// # Errors
    ///
    /// - `HoldError::HoldAlreadyUsed` if hold was already used for this piece
    /// - `HoldError::PieceCollision` if the swapped-in piece would collide
    pub fn try_hold(&mut self) -> Result<(), HoldError> {
        if self.hold_used {
            return Err(HoldError::HoldAlreadyUsed);
        }
        self.field.try_hold().map_err(HoldError::PieceCollision)?;
        self.hold_used = true;
        Ok(())
    }

    /// Performs a hard drop (instant drop to bottom) and completes the placement.
    ///
    /// Drops the piece as far as possible and locks it immediately.
    pub fn hard_drop_and_complete(&mut self) {
        while self.try_soft_drop().is_ok() {}
        self.complete_piece_drop();
    }

    /// Performs automatic gravity drop, completing placement if piece cannot drop further.
    ///
    /// Called automatically by `increment_frame` when the drop timer expires.
    pub fn auto_drop_and_complete(&mut self) {
        if self.try_soft_drop().is_ok() {
            return;
        }
        self.complete_piece_drop();
    }

    /// Internal method to complete a piece drop: lock, clear lines, spawn next piece.
    fn complete_piece_drop(&mut self) {
        self.block_board.fill_piece(self.field.falling_piece());
        let (cleared_lines, result) = self.field.complete_piece_drop();
        self.stats.complete_piece_drop(cleared_lines);
        self.hold_used = false;

        if result.is_err() {
            self.session_state = SessionState::GameOver;
            return;
        }
        assert_eq!(self.block_board.clear_lines(), cleared_lines);
    }
}
