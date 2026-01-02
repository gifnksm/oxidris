use oxidris_engine::{BitBoard, Piece};

use crate::board_analysis::BoardAnalysis;

#[derive(Debug)]
pub struct PlacementAnalysis {
    placement: Piece,
    cleared_lines: usize,
    board_analysis: BoardAnalysis,
}

impl PlacementAnalysis {
    #[must_use]
    pub fn from_board(board: &BitBoard, placement: Piece) -> Self {
        let mut board = board.clone();
        board.fill_piece(placement);
        let cleared_lines = board.clear_lines();

        Self {
            placement,
            cleared_lines,
            board_analysis: BoardAnalysis::from_board(&board),
        }
    }

    #[must_use]
    pub fn placement(&self) -> &Piece {
        &self.placement
    }

    #[must_use]
    pub fn cleared_lines(&self) -> usize {
        self.cleared_lines
    }

    #[must_use]
    pub fn board_analysis(&self) -> &BoardAnalysis {
        &self.board_analysis
    }
}
