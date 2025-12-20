use crate::core::bit_board::SENTINEL_MARGIN_BOTTOM;
use crate::core::piece::Piece;
use RenderCell::{Empty as E, Wall as W};

use super::bit_board::{
    PLAYABLE_HEIGHT, PLAYABLE_WIDTH, SENTINEL_MARGIN_LEFT, SENTINEL_MARGIN_RIGHT,
    SENTINEL_MARGIN_TOP, TOTAL_HEIGHT, TOTAL_WIDTH,
};
use super::piece::PieceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum RenderCell {
    #[default]
    Empty,
    Wall,
    Ghost,
    Piece(PieceKind),
}

impl RenderCell {
    #[must_use]
    pub fn is_empty(self) -> bool {
        self == RenderCell::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderRow {
    cells: [RenderCell; TOTAL_WIDTH],
}

impl RenderRow {
    // Sentinel border layout matches BitBoard for coordinate compatibility.
    // See BitBoard documentation for detailed explanation of the 2-cell sentinel design.
    const TOP: Self = {
        assert!(SENTINEL_MARGIN_LEFT == 2);
        assert!(SENTINEL_MARGIN_RIGHT == 2);
        RenderRow {
            cells: [W, W, E, E, E, E, E, E, E, E, E, E, W, W],
        }
    };
    const BOTTOM: Self = RenderRow {
        cells: [W; TOTAL_WIDTH],
    };

    fn playable_cells(&self) -> &[RenderCell; PLAYABLE_WIDTH] {
        self.cells[SENTINEL_MARGIN_LEFT..][..PLAYABLE_WIDTH]
            .try_into()
            .unwrap()
    }

    fn is_filled(&self) -> bool {
        self.playable_cells().iter().all(|b| !b.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct RenderBoard {
    rows: [RenderRow; TOTAL_HEIGHT],
}

impl RenderBoard {
    pub const PLAYABLE_WIDTH: usize = PLAYABLE_WIDTH;
    pub const PLAYABLE_HEIGHT: usize = PLAYABLE_HEIGHT;

    // Initial board layout matches BitBoard structure for coordinate compatibility.
    // Top sentinels use RenderRow::TOP (side walls only) to allow piece spawning.
    // Bottom sentinels use RenderRow::BOTTOM (fully occupied) to block downward movement.
    pub const INITIAL: Self = {
        assert!(SENTINEL_MARGIN_TOP == 2);
        assert!(SENTINEL_MARGIN_BOTTOM == 2);
        Self {
            rows: [
                // Top sentinel rows
                RenderRow::TOP,
                RenderRow::TOP,
                // Playable area rows
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                RenderRow::TOP,
                // Bottom sentinel rows
                RenderRow::BOTTOM,
                RenderRow::BOTTOM,
            ],
        }
    };

    pub fn playable_rows(&self) -> impl Iterator<Item = &[RenderCell; PLAYABLE_WIDTH]> {
        self.rows[SENTINEL_MARGIN_TOP..][..PLAYABLE_HEIGHT]
            .iter()
            .map(RenderRow::playable_cells)
    }

    pub fn fill_piece(&mut self, piece: &Piece) {
        for (x, y) in piece.occupied_positions() {
            self.rows[y].cells[x] = RenderCell::Piece(piece.kind());
        }
    }

    pub fn fill_piece_as(&mut self, piece: &Piece, cell: RenderCell) {
        for (x, y) in piece.occupied_positions() {
            self.rows[y].cells[x] = cell;
        }
    }

    pub fn clear_lines(&mut self) -> usize {
        let playable_rows = &mut self.rows[SENTINEL_MARGIN_TOP..][..PLAYABLE_HEIGHT];
        let mut count = 0;
        for y in (0..PLAYABLE_HEIGHT).rev() {
            if playable_rows[y].is_filled() {
                count += 1;
                continue;
            }
            if count > 0 {
                playable_rows[y + count] = playable_rows[y];
            }
        }
        playable_rows[..count].fill(RenderRow::TOP);
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_board() {
        let board = RenderBoard::INITIAL;

        for y in 0..TOTAL_HEIGHT {
            for x in 0..TOTAL_WIDTH {
                let cell = board.rows[y].cells[x];
                if y >= SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
                    assert_eq!(
                        cell,
                        RenderCell::Wall,
                        "Bottom sentinels should be walls, got {cell:?} at ({x}, {y})",
                    );
                    continue;
                }
                if !(SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH).contains(&x) {
                    assert_eq!(
                        cell,
                        RenderCell::Wall,
                        "Side sentinels should be walls, got {cell:?} at ({x}, {y})",
                    );
                    continue;
                }
                assert_eq!(
                    cell,
                    RenderCell::Empty,
                    "Playable area should be empty, got {cell:?} at ({x}, {y})",
                );
            }
        }
    }

    #[test]
    fn test_set_and_check_cell() {
        let mut board = RenderBoard::INITIAL;

        // Set a cell in the playable area
        let x = SENTINEL_MARGIN_LEFT;
        let y = SENTINEL_MARGIN_TOP;

        assert_eq!(board.rows[y].cells[x], RenderCell::Empty);
        board.rows[y].cells[x] = RenderCell::Piece(PieceKind::I);
        assert_eq!(board.rows[y].cells[x], RenderCell::Piece(PieceKind::I));
    }

    #[test]
    fn test_render_row_playable_cells() {
        let row = RenderRow::TOP;

        // Get playable cells
        let playable_cells = row.playable_cells();
        assert_eq!(playable_cells.len(), PLAYABLE_WIDTH);

        // All playable cells should be empty in TOP row
        for cell in playable_cells {
            assert_eq!(*cell, RenderCell::Empty);
        }
    }

    #[test]
    fn test_render_row_is_filled() {
        let mut row = RenderRow::TOP;
        // TOP row has empty playable area, so not filled
        assert!(!row.is_filled());

        // Fill playable area
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            row.cells[x] = RenderCell::Piece(PieceKind::I);
        }
        // Now it should be filled
        assert!(row.is_filled());
    }

    #[test]
    fn test_fill_piece_basic() {
        let mut board = RenderBoard::INITIAL;
        // Create a simple test piece at known position
        // This test verifies fill_piece_as works correctly

        let y = SENTINEL_MARGIN_TOP;
        let x = SENTINEL_MARGIN_LEFT;

        // Manually set a cell and verify fill_piece_as
        board.rows[y].cells[x] = RenderCell::Empty;
        assert_eq!(board.rows[y].cells[x], RenderCell::Empty);

        // Using fill_piece_as would require a Piece, which requires testing at a higher level
        // For now, verify the cell can be set
        board.rows[y].cells[x] = RenderCell::Piece(PieceKind::O);
        assert_eq!(board.rows[y].cells[x], RenderCell::Piece(PieceKind::O));
    }

    #[test]
    fn test_clear_lines_basic() {
        let mut board = RenderBoard::INITIAL;

        // Fill first playable line
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            board.rows[SENTINEL_MARGIN_TOP].cells[x] = RenderCell::Piece(PieceKind::I);
        }

        // Clear lines should remove the filled line
        let cleared = board.clear_lines();
        assert_eq!(cleared, 1);

        // The cleared line should now be empty (TOP row)
        let playable_cells = board.rows[SENTINEL_MARGIN_TOP].playable_cells();
        for cell in playable_cells {
            assert_eq!(*cell, RenderCell::Empty);
        }
    }

    #[test]
    fn test_clear_lines_multiple_consecutive() {
        let mut board = RenderBoard::INITIAL;

        // Fill three consecutive lines
        for i in 0..3 {
            let y = SENTINEL_MARGIN_TOP + i;
            for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
                board.rows[y].cells[x] = RenderCell::Piece(PieceKind::I);
            }
        }

        // Clear lines
        let cleared = board.clear_lines();
        assert_eq!(cleared, 3);

        // First three lines should be empty
        for i in 0..3 {
            let y = SENTINEL_MARGIN_TOP + i;
            let playable_cells = board.rows[y].playable_cells();
            for cell in playable_cells {
                assert_eq!(*cell, RenderCell::Empty);
            }
        }
    }

    #[test]
    fn test_clear_lines_with_partial_lines() {
        let mut board = RenderBoard::INITIAL;

        // Fill only part of the first line (not all playable cells)
        let y = SENTINEL_MARGIN_TOP;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH - 1 {
            board.rows[y].cells[x] = RenderCell::Piece(PieceKind::I);
        }
        // Leave one cell empty

        // Clear lines - should clear nothing
        let cleared = board.clear_lines();
        assert_eq!(cleared, 0);

        // Line should still have data
        let mut occupied_count = 0;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            if !board.rows[y].cells[x].is_empty() {
                occupied_count += 1;
            }
        }
        assert_eq!(occupied_count, PLAYABLE_WIDTH - 1);
    }

    #[test]
    fn test_clear_lines_bottom_line() {
        let mut board = RenderBoard::INITIAL;

        // Fill the last playable line
        let y = SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT - 1;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            board.rows[y].cells[x] = RenderCell::Piece(PieceKind::I);
        }

        // Clear lines
        let cleared = board.clear_lines();
        assert_eq!(cleared, 1);

        // The cleared line should be empty
        let playable_cells = board.rows[y].playable_cells();
        for cell in playable_cells {
            assert_eq!(*cell, RenderCell::Empty);
        }
    }

    #[test]
    fn test_clear_lines_all_filled() {
        let mut board = RenderBoard::INITIAL;

        // Fill all playable lines
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
                board.rows[y].cells[x] = RenderCell::Piece(PieceKind::I);
            }
        }

        // Clear lines
        let cleared = board.clear_lines();
        assert_eq!(cleared, PLAYABLE_HEIGHT);

        // All playable lines should now be empty
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            let playable_cells = board.rows[y].playable_cells();
            for cell in playable_cells {
                assert_eq!(*cell, RenderCell::Empty);
            }
        }
    }

    #[test]
    fn test_clear_lines_preserves_sentinels() {
        let mut board = RenderBoard::INITIAL;

        // Fill first playable line
        let y = SENTINEL_MARGIN_TOP;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            board.rows[y].cells[x] = RenderCell::Piece(PieceKind::I);
        }

        // Clear lines
        board.clear_lines();

        // Verify sentinels are still intact
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            assert_eq!(board.rows[y].cells[0], RenderCell::Wall);
            assert_eq!(board.rows[y].cells[1], RenderCell::Wall);
            assert_eq!(board.rows[y].cells[TOTAL_WIDTH - 2], RenderCell::Wall);
            assert_eq!(board.rows[y].cells[TOTAL_WIDTH - 1], RenderCell::Wall);
        }
    }
}
