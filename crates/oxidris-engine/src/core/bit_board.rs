use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::core::piece::Piece;

use super::{
    PLAYABLE_HEIGHT, PLAYABLE_WIDTH, SENTINEL_MARGIN_LEFT, SENTINEL_MARGIN_TOP, TOTAL_HEIGHT,
    TOTAL_WIDTH,
};

pub(super) const PIECE_SPAWN_X: usize = 5;
pub(super) const PIECE_SPAWN_Y: usize = 0;

// Bit masks for sentinel regions
// Left sentinel: bits 0-1 (x=0,1)
const LEFT_SENTINEL_MASK: u16 = 0b11;
// Right sentinel: bits 12-13 (x=12,13)
const RIGHT_SENTINEL_MASK: u16 = 0b11 << (SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH);
// Full sentinel mask (left + right)
const SENTINEL_MASK: u16 = LEFT_SENTINEL_MASK | RIGHT_SENTINEL_MASK;
// Full row (all cells occupied)
const FULL_ROW_MASK: u16 = (1 << TOTAL_WIDTH) - 1;
// Playable area mask (excluding sentinels)
const PLAYABLE_MASK: u16 = FULL_ROW_MASK & !SENTINEL_MASK;

/// Single row in the bit board representation.
/// Stores one row of the board as a u16 bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BitRow {
    bits: u16,
}

impl BitRow {
    pub const EMPTY: Self = Self {
        bits: SENTINEL_MASK,
    };
    pub const FULL_SENTINEL: Self = Self {
        bits: FULL_ROW_MASK,
    };

    /// Checks if the playable area is completely filled.
    #[inline]
    #[must_use]
    pub fn is_playable_filled(self) -> bool {
        (self.bits & PLAYABLE_MASK) == PLAYABLE_MASK
    }

    /// Checks if a cell at the given x-coordinate (in playable area) is occupied.
    #[inline]
    #[must_use]
    pub fn is_cell_occupied(self, x: usize) -> bool {
        let bit = 1 << x;
        (self.bits & bit) != 0
    }

    /// Checks if any cell in the given mask (shifted by x0) is occupied.
    #[inline]
    #[must_use]
    fn is_any_cell_occupied(self, x0: usize, mask: u16) -> bool {
        let bits = mask << x0;
        (self.bits & bits) != 0
    }

    /// Sets cells in the given mask (shifted by x0) as occupied.
    #[inline]
    fn occupy_cells(&mut self, x0: usize, mask: u16) {
        let bits = mask << x0;
        self.bits |= bits;
    }

    /// Iterates over all playable cells in the row, returning their occupied status.
    #[inline]
    pub fn iter_playable_cells(self) -> impl Iterator<Item = bool> {
        (SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH).map(move |x| {
            let bit = 1 << x;
            (self.bits & bit) != 0
        })
    }
}

/// `BitBoard` for fast collision detection and line clearing.
///
/// Each row is represented as a u16, where each bit represents a cell.
/// Bit layout (LSB to MSB, bit N corresponds to x=N):
/// - Bits 0-1: left sentinel (walls, x=0,1)
/// - Bits 2-11: playable area (10 cells, x=2-11)
/// - Bits 12-13: right sentinel (walls, x=12,13)
/// - Bits 14-15: unused (padding)
///
/// # Board layout with 2-cell sentinel borders
///
/// The board uses 2-cell sentinel borders on all sides instead of 1-cell borders.
/// This design accommodates the 4x4 grid positioning system used by all Tetris pieces.
///
/// ## Why 2 cells instead of 1?
///
/// All pieces use 4x4 grids for positioning and collision detection. The I-piece has
/// 2 consecutive empty columns adjacent to its cell, while other pieces have at most
/// 1 empty column in any direction. This means the I-piece needs more border space
/// to move naturally near board edges.
///
/// ## The vertical I-piece occupies this 4x4 grid pattern:
/// ```text
/// (. = empty, I = I-piece cell, W = wall)
///  [.  I  .  .]
///  [.  I  .  .]
///  [.  I  .  .]
///  [.  I  .  .]
/// ```
///
/// ## Problem with 1-cell border (showing 4x4 grid constraints):
/// ```text
///             0  1  2  3  4  5  6  7  8  9 10 11
/// Board:      W  .  .  .  .  .  .  .  .  .  .  W    ← 10-column playable area (column 1 to 10)
/// I-piece:   [W  I  .  .] .  .  .  .  .  .  .  W    ← Leftmost: I-piece cell at column 1
/// I-piece:    W  .  .  .  .  .  .  . [.  I  .  W]   ← Rightmost: I-piece cell at column 9, not 10
///                                          ^^^
///               The I-piece cannot reach right most columns due to 4x4 grid constraint
/// ```
///
/// ## Solution with 2-cell border:
/// ```text
///             0  1  2  3  4  5  6  7  8  9 10 11 12 13
/// Board:      W  W  .  .  .  .  .  .  .  .  .  .  W  W   ← 10-column playable area (column 2 to 11)
/// I-piece:    W [W  I  .  .] .  .  .  .  .  .  .  W  W   ← Leftmost: I-piece cell at column 2
/// I-piece:    W  W  .  .  .  .  .  .  .  . [.  I  W  W]  ← Rightmost: I-piece cell at column 11
/// ```
///
/// This allows full movement range while maintaining 4x4 grid collision detection.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct BitBoard {
    rows: [BitRow; TOTAL_HEIGHT],
}

impl BitBoard {
    pub const TOTAL_WIDTH: usize = TOTAL_WIDTH;
    pub const TOTAL_HEIGHT: usize = TOTAL_HEIGHT;
    pub const PLAYABLE_WIDTH: usize = PLAYABLE_WIDTH;
    pub const PLAYABLE_HEIGHT: usize = PLAYABLE_HEIGHT;
    pub const PLAYABLE_X_RANGE: Range<usize> =
        SENTINEL_MARGIN_LEFT..(SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH);
    pub const PLAYABLE_Y_RANGE: Range<usize> =
        SENTINEL_MARGIN_TOP..(SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT);

    pub const INITIAL: Self = {
        Self {
            rows: [
                // Top sentinel rows (only side sentinels, allow pieces to spawn above playable area)
                BitRow::EMPTY,
                BitRow::EMPTY,
                // Playable area rows (only sentinels on sides)
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                BitRow::EMPTY,
                // Bottom sentinel rows (fully occupied to block downward movement)
                BitRow::FULL_SENTINEL,
                BitRow::FULL_SENTINEL,
            ],
        }
    };

    /// Returns a reference to a playable row by index.
    #[must_use]
    pub fn playable_row(&self, y: usize) -> BitRow {
        self.rows[y + SENTINEL_MARGIN_TOP]
    }

    /// Returns an iterator over the playable rows.
    pub fn playable_rows(&self) -> impl Iterator<Item = BitRow> + '_ {
        self.rows[SENTINEL_MARGIN_TOP..][..PLAYABLE_HEIGHT]
            .iter()
            .copied()
    }

    /// Checks if the piece collides with occupied cells.
    #[must_use]
    pub fn is_colliding(&self, piece: Piece) -> bool {
        let x0 = piece.position().x();
        let y0 = piece.position().y();
        for (mask, row) in piece.mask().into_iter().zip(&self.rows[y0..]) {
            if row.is_any_cell_occupied(x0, mask) {
                return true;
            }
        }
        false
    }

    /// Fills the piece cells on the board.
    pub fn fill_piece(&mut self, piece: Piece) {
        let x0 = piece.position().x();
        let y0 = piece.position().y();
        for (mask, row) in piece.mask().into_iter().zip(&mut self.rows[y0..]) {
            row.occupy_cells(x0, mask);
        }
    }

    /// Clears filled lines and returns the number of lines cleared.
    ///
    /// A line is considered filled when all playable cells are occupied.
    pub fn clear_lines(&mut self) -> usize {
        let playable_rows = &mut self.rows[SENTINEL_MARGIN_TOP..][..PLAYABLE_HEIGHT];
        let mut count = 0;

        for y in (0..PLAYABLE_HEIGHT).rev() {
            if playable_rows[y].is_playable_filled() {
                count += 1;
                continue;
            }
            if count > 0 {
                playable_rows[y + count] = playable_rows[y];
            }
        }

        // Fill cleared lines at the top with empty rows (only sentinels)
        playable_rows[..count].fill(BitRow::EMPTY);
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_occupied(board: &BitBoard, x: usize, y: usize) -> bool {
        board.rows[y].is_cell_occupied(x)
    }

    fn occupy_cell(board: &mut BitBoard, x: usize, y: usize) {
        board.rows[y].occupy_cells(x, 0b1);
    }

    #[test]
    fn test_initial_board() {
        let board = BitBoard::INITIAL;

        for y in 0..TOTAL_HEIGHT {
            for x in 0..TOTAL_WIDTH {
                let cell = board.rows[y].is_cell_occupied(x);
                if y >= SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
                    assert!(
                        cell,
                        "Bottom sentinels should be occupied, got {cell:?} at ({x}, {y})",
                    );
                    continue;
                }
                if !(SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH).contains(&x) {
                    assert!(
                        cell,
                        "Side sentinels should be occupied, got {cell:?} at ({x}, {y})",
                    );
                    continue;
                }
                assert!(
                    !cell,
                    "Playable area should not be occupied, got {cell:?} at ({x}, {y})",
                );
            }
        }
    }

    #[test]
    fn test_occupy_and_check_cell() {
        let mut board = BitBoard::INITIAL;

        // Occupy a cell in the playable area
        let x = SENTINEL_MARGIN_LEFT;
        let y = SENTINEL_MARGIN_TOP;

        assert!(!is_occupied(&board, x, y));
        occupy_cell(&mut board, x, y);
        assert!(is_occupied(&board, x, y));
    }

    #[test]
    fn test_bit_row_set_and_check() {
        let mut row = BitRow::EMPTY;

        // Set a cell and check
        let x = SENTINEL_MARGIN_LEFT;
        assert!(!row.is_cell_occupied(x));
        row.occupy_cells(x, 0b1);
        assert!(row.is_cell_occupied(x));

        // Check that other cells are not occupied
        let x_other = SENTINEL_MARGIN_LEFT + 1;
        assert!(!row.is_cell_occupied(x_other));
    }

    #[test]
    fn test_bit_row_is_playable_filled() {
        let mut row = BitRow::EMPTY;

        // Initially not filled
        assert!(!row.is_playable_filled());

        // Fill all playable cells
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            row.occupy_cells(x, 0b1);
        }

        // Now it should be filled
        assert!(row.is_playable_filled());
    }

    #[test]
    fn test_clear_lines_single_line() {
        let mut board = BitBoard::INITIAL;

        // Fill the first playable line
        let y = SENTINEL_MARGIN_TOP;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            occupy_cell(&mut board, x, y);
        }

        // Verify it's filled
        assert!(board.rows[y].is_playable_filled());

        // Clear lines
        let cleared = board.clear_lines();
        assert_eq!(cleared, 1);

        // The first playable line should now be empty (SENTINEL_MASK)
        assert_eq!(board.rows[y].bits, SENTINEL_MASK);

        // Verify no other lines are filled
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            assert!(!board.rows[y].is_playable_filled());
        }
    }

    #[test]
    fn test_clear_lines_multiple_consecutive() {
        let mut board = BitBoard::INITIAL;

        // Fill three consecutive lines
        for i in 0..3 {
            let y = SENTINEL_MARGIN_TOP + i;
            for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
                occupy_cell(&mut board, x, y);
            }
        }

        // Clear lines
        let cleared = board.clear_lines();
        assert_eq!(cleared, 3);

        // First three lines should be empty
        for i in 0..3 {
            let y = SENTINEL_MARGIN_TOP + i;
            assert!(!board.rows[y].is_playable_filled());
            assert_eq!(board.rows[y].bits, SENTINEL_MASK);
        }

        // Fourth line should still be empty (not filled before clear)
        let y = SENTINEL_MARGIN_TOP + 3;
        assert!(!board.rows[y].is_playable_filled());
    }

    #[test]
    fn test_clear_lines_with_partial_lines() {
        let mut board = BitBoard::INITIAL;

        // Fill only part of the first line (not all playable cells)
        let y = SENTINEL_MARGIN_TOP;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH - 1 {
            occupy_cell(&mut board, x, y);
        }
        // Leave one cell empty

        // Clear lines - should clear nothing
        let cleared = board.clear_lines();
        assert_eq!(cleared, 0);

        // Line should still have data
        let mut occupied_count = 0;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            if board.rows[y].is_cell_occupied(x) {
                occupied_count += 1;
            }
        }
        assert_eq!(occupied_count, PLAYABLE_WIDTH - 1);
    }

    #[test]
    fn test_clear_lines_bottom_lines() {
        let mut board = BitBoard::INITIAL;

        // Fill the last playable line (line 19, which is SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT - 1)
        let y = SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT - 1;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            occupy_cell(&mut board, x, y);
        }

        // Clear lines
        let cleared = board.clear_lines();
        assert_eq!(cleared, 1);

        // The cleared line should be empty
        assert!(!board.rows[y].is_playable_filled());
    }

    #[test]
    fn test_clear_lines_all_filled() {
        let mut board = BitBoard::INITIAL;

        // Fill all playable lines
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
                occupy_cell(&mut board, x, y);
            }
        }

        // Clear lines
        let cleared = board.clear_lines();
        assert_eq!(cleared, PLAYABLE_HEIGHT);

        // All playable lines should now be empty
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            assert!(!board.rows[y].is_playable_filled());
            assert_eq!(board.rows[y].bits, SENTINEL_MASK);
        }
    }

    #[test]
    fn test_clear_lines_preserves_sentinels() {
        let mut board = BitBoard::INITIAL;

        // Fill first playable line
        let y = SENTINEL_MARGIN_TOP;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            occupy_cell(&mut board, x, y);
        }

        // Clear lines
        board.clear_lines();

        // Verify sentinels are still intact
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            // Check left sentinel
            assert!(board.rows[y].is_cell_occupied(0));
            assert!(board.rows[y].is_cell_occupied(1));

            // Check right sentinel
            assert!(board.rows[y].is_cell_occupied(TOTAL_WIDTH - 2));
            assert!(board.rows[y].is_cell_occupied(TOTAL_WIDTH - 1));
        }
    }
}
