use crate::core::{
    bit_board::BitBoard,
    piece::{Piece, PieceGenerator, PieceKind},
};

const SCORE_TABLE: [usize; 5] = [0, 100, 300, 500, 800];

#[derive(Debug, Clone)]
pub struct GameState {
    board: BitBoard,
    falling_piece: Piece,
    held_piece: Option<PieceKind>,
    hold_used: bool,
    piece_generator: PieceGenerator,
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
    pub fn new() -> Self {
        let first_piece = PieceKind::I; // dummy initial value
        let mut game = Self {
            board: BitBoard::INITIAL,
            falling_piece: Piece::new(first_piece),
            held_piece: None,
            hold_used: false,
            piece_generator: PieceGenerator::new(),
            score: 0,
            completed_pieces: 0,
            total_cleared_lines: 0,
            line_cleared_counter: [0; 5],
        };
        game.begin_next_piece_fall();
        game
    }

    pub fn level(&self) -> usize {
        self.total_cleared_lines / 10
    }

    pub fn total_cleared_lines(&self) -> usize {
        self.total_cleared_lines
    }

    pub fn completed_pieces(&self) -> usize {
        self.completed_pieces
    }

    pub fn line_cleared_counter(&self) -> &[usize; 5] {
        &self.line_cleared_counter
    }

    pub fn score(&self) -> usize {
        self.score
    }

    pub fn board(&self) -> &BitBoard {
        &self.board
    }

    pub fn falling_piece(&self) -> &Piece {
        &self.falling_piece
    }

    pub fn set_falling_piece(&mut self, piece: Piece) -> Result<(), ()> {
        if self.board.is_colliding(&piece) {
            return Err(());
        }
        self.falling_piece = piece;
        Ok(())
    }

    pub fn set_falling_piece_unchecked(&mut self, piece: Piece) {
        self.falling_piece = piece;
    }

    pub fn held_piece(&self) -> Option<PieceKind> {
        self.held_piece
    }

    pub fn is_hold_used(&self) -> bool {
        self.hold_used
    }

    pub fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.piece_generator.next_pieces()
    }

    pub fn simulate_drop_position(&self) -> Piece {
        self.falling_piece.simulate_drop_position(&self.board)
    }

    pub fn try_hold(&mut self) -> Result<(), ()> {
        if self.hold_used {
            return Err(());
        }
        if let Some(held_piece) = self.held_piece {
            let piece = Piece::new(held_piece);
            if self.board.is_colliding(&piece) {
                return Err(());
            }
            self.held_piece = Some(self.falling_piece.kind());
            self.falling_piece = piece;
        } else {
            self.held_piece = Some(self.falling_piece.kind());
            self.begin_next_piece_fall();
        }
        self.hold_used = true;
        Ok(())
    }

    pub fn complete_piece_drop(&mut self) -> Result<usize, ()> {
        self.board.fill_piece(&self.falling_piece);
        let cleared_lines = self.board.clear_lines();
        self.score += SCORE_TABLE[cleared_lines];
        self.total_cleared_lines += cleared_lines;
        self.line_cleared_counter[cleared_lines] += 1;
        self.completed_pieces += 1;

        self.begin_next_piece_fall();
        if self.board.is_colliding(&self.falling_piece) {
            return Err(());
        }

        self.hold_used = false;
        Ok(cleared_lines)
    }

    fn begin_next_piece_fall(&mut self) {
        self.falling_piece = Piece::new(self.piece_generator.pop_next());
    }
}
