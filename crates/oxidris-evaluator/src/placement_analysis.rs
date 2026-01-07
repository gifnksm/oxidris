//! Analysis of board state after piece placement.
//!
//! This module provides [`PlacementAnalysis`], which encapsulates the result of placing
//! a piece on the board: the cleared lines count and the resulting board state metrics.
//!
//! # Design
//!
//! `PlacementAnalysis` combines two pieces of information:
//!
//! 1. **Line clears** - How many lines were cleared by this placement
//! 2. **Board state** - The resulting board metrics via [`BoardAnalysis`]
//!
//! This unified analysis is used by board features to extract both placement-specific
//! information (lines cleared) and board-state information (holes, height, etc.).
//!
//! # Usage
//!
//! Features create `PlacementAnalysis` to evaluate a potential piece placement:
//!
//! ```rust,ignore
//! let analysis = PlacementAnalysis::from_board(&board, placement);
//! let lines = analysis.cleared_lines();
//! let board_analysis = analysis.board_analysis();
//! let holes = board_analysis.num_holes();
//! ```
//!
//! The `BoardAnalysis` inside provides lazy-evaluated metrics, making it efficient
//! even when only a subset of metrics are needed.

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
    pub fn from_board(before_placement: &BitBoard, placement: Piece) -> Self {
        let mut board = before_placement.clone();
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
