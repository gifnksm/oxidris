use crate::{
    HoldError, PieceCollisionError,
    core::{
        bit_board::BitBoard,
        piece::{Piece, PieceKind},
    },
};

use super::piece_generator::PieceBuffer;

const SCORE_TABLE: [usize; 5] = [0, 100, 300, 500, 800];

#[derive(Debug, Clone)]
pub struct GameState {
    board: BitBoard,
    falling_piece: Piece,
    hold_used: bool,
    piece_buffer: PieceBuffer,
    score: usize,
    completed_pieces: usize,
    total_cleared_lines: usize,
    line_cleared_counter: [usize; 5],
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    #[must_use]
    pub fn new() -> Self {
        let mut piece_buffer = PieceBuffer::new();
        let falling_piece = Piece::new(piece_buffer.pop_next());
        Self {
            board: BitBoard::INITIAL,
            falling_piece,
            hold_used: false,
            piece_buffer,
            score: 0,
            completed_pieces: 0,
            total_cleared_lines: 0,
            line_cleared_counter: [0; 5],
        }
    }

    #[must_use]
    pub fn level(&self) -> usize {
        self.total_cleared_lines / 10
    }

    #[must_use]
    pub fn total_cleared_lines(&self) -> usize {
        self.total_cleared_lines
    }

    #[must_use]
    pub fn completed_pieces(&self) -> usize {
        self.completed_pieces
    }

    #[must_use]
    pub fn line_cleared_counter(&self) -> &[usize; 5] {
        &self.line_cleared_counter
    }

    #[must_use]
    pub fn score(&self) -> usize {
        self.score
    }

    #[must_use]
    pub fn board(&self) -> &BitBoard {
        &self.board
    }

    #[must_use]
    pub fn falling_piece(&self) -> &Piece {
        &self.falling_piece
    }

    pub fn set_falling_piece(&mut self, piece: Piece) -> Result<(), PieceCollisionError> {
        if self.board.is_colliding(&piece) {
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

    #[must_use]
    pub fn is_hold_used(&self) -> bool {
        self.hold_used
    }

    pub fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.piece_buffer.next_pieces()
    }

    #[must_use]
    pub fn simulate_drop_position(&self) -> Piece {
        self.falling_piece.simulate_drop_position(&self.board)
    }

    pub fn try_hold(&mut self) -> Result<(), HoldError> {
        if self.hold_used {
            return Err(HoldError::HoldAlreadyUsed);
        }
        if let Some(piece) = self.piece_buffer.held_piece() {
            let piece = Piece::new(piece);
            if self.board.is_colliding(&piece) {
                return Err(HoldError::PieceCollision(PieceCollisionError));
            }
            self.piece_buffer.swap_hold(self.falling_piece.kind());
            self.falling_piece = piece;
        } else {
            self.piece_buffer.swap_hold(self.falling_piece.kind());
            self.falling_piece = Piece::new(self.piece_buffer.pop_next());
        }
        self.hold_used = true;
        Ok(())
    }

    pub fn complete_piece_drop(&mut self) -> Result<usize, PieceCollisionError> {
        self.board.fill_piece(&self.falling_piece);
        let cleared_lines = self.board.clear_lines();
        self.score += SCORE_TABLE[cleared_lines];
        self.total_cleared_lines += cleared_lines;
        self.line_cleared_counter[cleared_lines] += 1;
        self.completed_pieces += 1;

        self.falling_piece = Piece::new(self.piece_buffer.pop_next());
        if self.board.is_colliding(&self.falling_piece) {
            return Err(PieceCollisionError);
        }

        self.hold_used = false;
        Ok(cleared_lines)
    }
}
