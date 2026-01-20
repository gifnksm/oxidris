use rand::Rng as _;

use super::piece_buffer::PieceBuffer;
use crate::{
    CompletePieceDropError, PieceCollisionError, PieceSeed,
    core::{
        bit_board::BitBoard,
        piece::{Piece, PieceKind},
    },
};

/// Single-turn game state for Tetris.
///
/// `GameField` represents the state of a Tetris game at a specific point in time,
/// including the board, the currently falling piece, and the piece queue/hold system.
///
/// This is a low-level API for direct state manipulation. For turn-based gameplay
/// with statistics tracking, use [`GameSession`](super::GameSession).
///
/// # Structure
///
/// - **Board**: 10×20 playable area with piece blocks
/// - **Falling piece**: Currently active piece that can be moved/rotated
/// - **Piece buffer**: 7-bag piece generator with hold system
///
/// # Example
///
/// ```
/// use oxidris_engine::GameField;
///
/// let mut field = GameField::new();
///
/// // Manipulate the falling piece
/// if let Some(piece) = field.falling_piece().left() {
///     field.set_falling_piece(piece).ok();
/// }
///
/// // Complete the placement
/// let (lines_cleared, result) = field.complete_piece_drop();
/// ```
#[derive(Debug, Clone)]
pub struct GameField {
    board: BitBoard,
    falling_piece: Piece,
    piece_buffer: PieceBuffer,
}

impl Default for GameField {
    fn default() -> Self {
        Self::new()
    }
}

impl GameField {
    /// Creates a new game field with an empty board and first piece spawned.
    ///
    /// The piece buffer is initialized with a random seed, and the first piece
    /// is drawn from the 7-bag system. For deterministic piece generation, use
    /// [`Self::with_seed`] instead.
    #[must_use]
    pub fn new() -> Self {
        Self::with_seed(rand::rng().random())
    }

    /// Like [`Self::new`], but with a specific seed for deterministic piece generation.
    #[must_use]
    pub fn with_seed(seed: PieceSeed) -> Self {
        let mut piece_buffer = PieceBuffer::with_seed(seed);
        let falling_piece = Piece::new(piece_buffer.pop_next());
        Self {
            board: BitBoard::INITIAL,
            falling_piece,
            piece_buffer,
        }
    }

    /// Returns a reference to the current board state.
    #[must_use]
    pub fn board(&self) -> &BitBoard {
        &self.board
    }

    /// Returns the currently falling piece.
    #[must_use]
    pub fn falling_piece(&self) -> Piece {
        self.falling_piece
    }

    /// Sets the falling piece to a new position/rotation, checking for collisions.
    ///
    /// This is the primary way to manipulate the falling piece. Use this after
    /// computing a new position (e.g., `piece.left()`, `piece.rotated_right()`).
    ///
    /// # Errors
    ///
    /// Returns `PieceCollisionError` if the new piece position collides with the board.
    pub fn set_falling_piece(&mut self, piece: Piece) -> Result<(), PieceCollisionError> {
        if self.board.is_colliding(piece) {
            return Err(PieceCollisionError);
        }
        self.falling_piece = piece;
        Ok(())
    }

    /// Sets the falling piece without collision checking.
    ///
    /// # Safety
    ///
    /// Caller must ensure the piece does not collide with the board.
    /// Use [`set_falling_piece`](Self::set_falling_piece) for safe manipulation.
    pub fn set_falling_piece_unchecked(&mut self, piece: Piece) {
        self.falling_piece = piece;
    }

    /// Returns the currently held piece, if any.
    #[must_use]
    pub fn held_piece(&self) -> Option<PieceKind> {
        self.piece_buffer.held_piece()
    }

    /// Returns an iterator over the upcoming pieces in the queue.
    pub fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.piece_buffer.next_pieces()
    }

    /// Simulates a hard drop and returns the final position without modifying state.
    ///
    /// This is useful for previewing where a piece will land or for AI evaluation.
    #[must_use]
    pub fn simulate_drop_position(&self) -> Piece {
        self.falling_piece.simulate_drop_position(&self.board)
    }

    /// Checks if the hold operation is valid (piece would not collide after swap).
    #[must_use]
    pub fn can_hold(&self) -> bool {
        let piece = self.piece_buffer.peek_hold_result();
        !self.board.is_colliding(Piece::new(piece))
    }

    /// Returns what the falling piece would be after a hold operation, without executing it.
    #[must_use]
    pub fn peek_falling_piece_after_hold(&self) -> Piece {
        Piece::new(self.piece_buffer.peek_hold_result())
    }

    /// Attempts to hold the current piece and swap it with the held piece (or next piece).
    ///
    /// # Behavior
    ///
    /// - If a piece is already held, swaps current piece with held piece
    /// - If no piece is held, stores current piece and draws from next queue
    /// - New piece spawns at standard spawn position
    ///
    /// # Errors
    ///
    /// Returns `PieceCollisionError` if the swapped-in piece collides at spawn.
    pub fn try_hold(&mut self) -> Result<(), PieceCollisionError> {
        if !self.can_hold() {
            return Err(PieceCollisionError);
        }

        let next_piece = self.piece_buffer.hold(self.falling_piece.kind());
        self.falling_piece = Piece::new(next_piece);

        Ok(())
    }

    /// Completes the current piece placement: locks piece, clears lines, spawns next piece.
    ///
    /// This is called after the piece has been moved to its final position (typically
    /// after a hard drop). The sequence is:
    ///
    /// 1. Lock the falling piece into the board
    /// 2. Clear any completed lines
    /// 3. Spawn the next piece from the queue
    ///
    /// # Returns
    ///
    /// A tuple of:
    /// - Number of lines cleared (0-4)
    /// - `Ok(())` if the next piece spawned successfully
    /// - `Err(CompletePieceDropError::NewPieceCollision)` if game over (top-out)
    pub fn complete_piece_drop(&mut self) -> (usize, Result<(), CompletePieceDropError>) {
        self.board.fill_piece(self.falling_piece);
        let cleared_lines = self.board.clear_lines();

        self.falling_piece = Piece::new(self.piece_buffer.pop_next());
        if self.board.is_colliding(self.falling_piece) {
            return (
                cleared_lines,
                Err(CompletePieceDropError::NewPieceCollision),
            );
        }

        (cleared_lines, Ok(()))
    }
}
