use crate::{
    CompletePieceDropError, PieceCollisionError,
    core::{
        bit_board::BitBoard,
        piece::{Piece, PieceKind},
    },
};

use super::piece_buffer::PieceBuffer;

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
    #[must_use]
    pub fn new() -> Self {
        let mut piece_buffer = PieceBuffer::new();
        let falling_piece = Piece::new(piece_buffer.pop_next());
        Self {
            board: BitBoard::INITIAL,
            falling_piece,
            piece_buffer,
        }
    }

    #[must_use]
    pub fn board(&self) -> &BitBoard {
        &self.board
    }

    #[must_use]
    pub fn falling_piece(&self) -> Piece {
        self.falling_piece
    }

    pub fn set_falling_piece(&mut self, piece: Piece) -> Result<(), PieceCollisionError> {
        if self.board.is_colliding(piece) {
            return Err(PieceCollisionError);
        }
        self.falling_piece = piece;
        Ok(())
    }

    pub fn set_falling_piece_unchecked(&mut self, piece: Piece) {
        self.falling_piece = piece;
    }

    #[must_use]
    pub fn held_piece(&self) -> Option<PieceKind> {
        self.piece_buffer.held_piece()
    }

    pub fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.piece_buffer.next_pieces()
    }

    #[must_use]
    pub fn simulate_drop_position(&self) -> Piece {
        self.falling_piece.simulate_drop_position(&self.board)
    }

    #[must_use]
    pub fn can_hold(&self) -> bool {
        let piece = self.piece_buffer.peek_hold_result();
        !self.board.is_colliding(Piece::new(piece))
    }

    #[must_use]
    pub fn peek_falling_piece_after_hold(&self) -> Piece {
        Piece::new(self.piece_buffer.peek_hold_result())
    }

    pub fn try_hold(&mut self) -> Result<(), PieceCollisionError> {
        if !self.can_hold() {
            return Err(PieceCollisionError);
        }

        let next_piece = self.piece_buffer.hold(self.falling_piece.kind());
        self.falling_piece = Piece::new(next_piece);

        Ok(())
    }

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
