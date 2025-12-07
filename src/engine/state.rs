use crate::core::{
    board::Board,
    piece::{Piece, PieceGenerator, PieceKind},
};

const SCORE_TABLE: [usize; 5] = [0, 1, 5, 25, 100];

#[derive(Debug, Clone)]
pub(crate) struct GameState {
    board: Board,
    falling_piece: Piece,
    held_piece: Option<PieceKind>,
    hold_used: bool,
    piece_generator: PieceGenerator,
    score: usize,
    cleared_lines: usize,
}

impl GameState {
    pub(crate) fn new() -> Self {
        let first_piece = PieceKind::I; // dummy initial value
        let mut game = Self {
            board: Board::INITIAL,
            falling_piece: Piece::new(first_piece),
            held_piece: None,
            hold_used: false,
            piece_generator: PieceGenerator::new(),
            score: 0,
            cleared_lines: 0,
        };
        game.begin_next_piece_fall();
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

    pub(crate) fn board(&self) -> &Board {
        &self.board
    }

    pub(crate) fn falling_piece(&self) -> &Piece {
        &self.falling_piece
    }

    pub(crate) fn set_falling_piece(&mut self, piece: Piece) -> Result<(), ()> {
        if self.board.is_colliding(&piece) {
            return Err(());
        }
        self.falling_piece = piece;
        Ok(())
    }

    pub(crate) fn set_falling_piece_unchecked(&mut self, piece: Piece) {
        self.falling_piece = piece;
    }

    pub(crate) fn held_piece(&self) -> Option<PieceKind> {
        self.held_piece
    }

    pub(crate) fn is_hold_used(&self) -> bool {
        self.hold_used
    }

    pub(crate) fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.piece_generator.next_pieces()
    }

    pub(crate) fn simulate_drop_position(&self) -> Piece {
        self.falling_piece.simulate_drop_position(&self.board)
    }

    pub(crate) fn try_hold(&mut self) -> Result<(), ()> {
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

    pub(crate) fn complete_piece_drop(&mut self) -> Result<(), ()> {
        self.board.fill_piece(&self.falling_piece);
        let line = self.board.clear_lines();
        self.score += SCORE_TABLE[line];
        self.cleared_lines += line;

        self.begin_next_piece_fall();
        if self.board.is_colliding(&self.falling_piece) {
            return Err(());
        }

        self.hold_used = false;
        Ok(())
    }

    fn begin_next_piece_fall(&mut self) {
        self.falling_piece = Piece::new(self.piece_generator.pop_next());
    }
}
