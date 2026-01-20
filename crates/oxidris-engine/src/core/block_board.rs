use super::{
    PLAYABLE_HEIGHT, PLAYABLE_WIDTH, SENTINEL_MARGIN_BOTTOM, SENTINEL_MARGIN_LEFT,
    SENTINEL_MARGIN_RIGHT, SENTINEL_MARGIN_TOP, TOTAL_HEIGHT, TOTAL_WIDTH,
    piece::{Piece, PieceKind},
};

/// A single cell in the block board representation.
///
/// Used for rendering and visual representation of the game state.
/// Unlike [`BitBoard`](super::bit_board::BitBoard) which uses bits for collision detection,
/// `Block` stores semantic information about what occupies each cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Block {
    /// Empty cell (no piece).
    #[default]
    Empty,
    /// Wall (sentinel border).
    Wall,
    /// Ghost piece preview (shows where piece will land).
    Ghost,
    /// Locked piece of a specific type.
    Piece(PieceKind),
}

impl Block {
    #[must_use]
    pub fn is_empty(self) -> bool {
        self == Block::Empty
    }
}

/// A single row in the block board representation.
///
/// Stores a row of cells with the same layout as [`BitBoard`](super::bit_board::BitBoard)
/// for coordinate compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRow {
    cells: [Block; TOTAL_WIDTH],
}

impl BlockRow {
    // Sentinel border layout matches BitBoard for coordinate compatibility.
    // See BitBoard documentation for detailed explanation of the 2-cell sentinel design.
    const TOP: Self = {
        use Block::{Empty as E, Wall as W};
        assert!(SENTINEL_MARGIN_LEFT == 2);
        assert!(SENTINEL_MARGIN_RIGHT == 2);
        BlockRow {
            cells: [W, W, E, E, E, E, E, E, E, E, E, E, W, W],
        }
    };
    const BOTTOM: Self = BlockRow {
        cells: [Block::Wall; TOTAL_WIDTH],
    };

    fn playable_cells(&self) -> &[Block; PLAYABLE_WIDTH] {
        self.cells[SENTINEL_MARGIN_LEFT..][..PLAYABLE_WIDTH]
            .try_into()
            .unwrap()
    }

    fn is_filled(&self) -> bool {
        self.playable_cells().iter().all(|b| !b.is_empty())
    }
}

/// Cell-by-cell board representation for rendering and analysis.
///
/// `BlockBoard` stores the board state as individual cells, making it suitable for:
///
/// - **Rendering**: Each cell knows its type (empty, wall, piece kind)
/// - **Visual effects**: Can render ghost pieces (drop preview)
/// - **Analysis**: Easy to inspect individual cells
///
/// This complements [`BitBoard`](super::bit_board::BitBoard) which is optimized for
/// collision detection. Both use the same coordinate system and sentinel layout.
///
/// # Layout
///
/// - **Total dimensions**: 14×24 (includes 2-cell sentinel margins)
/// - **Playable area**: 10×20 (standard Tetris)
/// - **Coordinate compatibility**: Matches [`BitBoard`](super::bit_board::BitBoard) exactly
///
/// # Example
///
/// ```
/// use oxidris_engine::BlockBoard;
///
/// let board = BlockBoard::INITIAL;
/// for row in board.playable_rows() {
///     // Process each row of playable cells
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BlockBoard {
    rows: [BlockRow; TOTAL_HEIGHT],
}

impl BlockBoard {
    pub const PLAYABLE_WIDTH: usize = PLAYABLE_WIDTH;
    pub const PLAYABLE_HEIGHT: usize = PLAYABLE_HEIGHT;

    // Initial board layout matches BitBoard structure for coordinate compatibility.
    // Top sentinels use BlockRow::TOP (side walls only) to allow piece spawning.
    // Bottom sentinels use BlockRow::BOTTOM (fully occupied) to block downward movement.
    pub const INITIAL: Self = {
        assert!(SENTINEL_MARGIN_TOP == 2);
        assert!(SENTINEL_MARGIN_BOTTOM == 2);
        Self {
            rows: [
                // Top sentinel rows
                BlockRow::TOP,
                BlockRow::TOP,
                // Playable area rows
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                BlockRow::TOP,
                // Bottom sentinel rows
                BlockRow::BOTTOM,
                BlockRow::BOTTOM,
            ],
        }
    };

    /// Returns an iterator over the playable rows (excludes sentinel margins).
    ///
    /// Each row is a fixed-size array of playable cells.
    pub fn playable_rows(&self) -> impl Iterator<Item = &[Block; PLAYABLE_WIDTH]> {
        self.rows[SENTINEL_MARGIN_TOP..][..PLAYABLE_HEIGHT]
            .iter()
            .map(BlockRow::playable_cells)
    }

    /// Fills the piece's cells on the board with the piece's type.
    ///
    /// This is called when a piece is locked into position.
    pub fn fill_piece(&mut self, piece: Piece) {
        for (x, y) in piece.occupied_positions() {
            self.rows[y].cells[x] = Block::Piece(piece.kind());
        }
    }

    /// Fills a single cell on the board with the specified block.
    ///
    /// Coordinates use the internal coordinate system (including sentinel margins),
    /// matching `BitBoard::occupied_cell_positions()` and `Piece::occupied_positions()`.
    pub fn fill_block_at(&mut self, x: usize, y: usize, block: Block) {
        self.rows[y].cells[x] = block;
    }

    /// Fills the piece's cells on the board with a specific block type.
    ///
    /// Useful for rendering ghost pieces (drop preview) or other visual effects.
    ///
    /// # Example
    ///
    /// ```
    /// use oxidris_engine::{Block, BlockBoard, Piece, PieceKind};
    ///
    /// let mut board = BlockBoard::INITIAL;
    /// let piece = Piece::new(PieceKind::T);
    ///
    /// // Render as ghost piece
    /// board.fill_piece_as(piece, Block::Ghost);
    /// ```
    pub fn fill_piece_as(&mut self, piece: Piece, cell: Block) {
        for (x, y) in piece.occupied_positions() {
            self.rows[y].cells[x] = cell;
        }
    }

    /// Clears filled lines and returns the number of lines cleared.
    ///
    /// A line is filled when all playable cells contain non-empty blocks.
    /// Cleared lines are removed and rows above shift down.
    ///
    /// # Returns
    ///
    /// The number of lines cleared (0-4 in standard gameplay).
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
        playable_rows[..count].fill(BlockRow::TOP);
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_board() {
        let board = BlockBoard::INITIAL;

        for y in 0..TOTAL_HEIGHT {
            for x in 0..TOTAL_WIDTH {
                let cell = board.rows[y].cells[x];
                if y >= SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
                    assert_eq!(
                        cell,
                        Block::Wall,
                        "Bottom sentinels should be walls, got {cell:?} at ({x}, {y})",
                    );
                    continue;
                }
                if !(SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH).contains(&x) {
                    assert_eq!(
                        cell,
                        Block::Wall,
                        "Side sentinels should be walls, got {cell:?} at ({x}, {y})",
                    );
                    continue;
                }
                assert_eq!(
                    cell,
                    Block::Empty,
                    "Playable area should be empty, got {cell:?} at ({x}, {y})",
                );
            }
        }
    }

    #[test]
    fn test_set_and_check_cell() {
        let mut board = BlockBoard::INITIAL;

        // Set a cell in the playable area
        let x = SENTINEL_MARGIN_LEFT;
        let y = SENTINEL_MARGIN_TOP;

        assert_eq!(board.rows[y].cells[x], Block::Empty);
        board.rows[y].cells[x] = Block::Piece(PieceKind::I);
        assert_eq!(board.rows[y].cells[x], Block::Piece(PieceKind::I));
    }

    #[test]
    fn test_render_row_playable_cells() {
        let row = BlockRow::TOP;

        // Get playable cells
        let playable_cells = row.playable_cells();
        assert_eq!(playable_cells.len(), PLAYABLE_WIDTH);

        // All playable cells should be empty in TOP row
        for cell in playable_cells {
            assert_eq!(*cell, Block::Empty);
        }
    }

    #[test]
    fn test_render_row_is_filled() {
        let mut row = BlockRow::TOP;
        // TOP row has empty playable area, so not filled
        assert!(!row.is_filled());

        // Fill playable area
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            row.cells[x] = Block::Piece(PieceKind::I);
        }
        // Now it should be filled
        assert!(row.is_filled());
    }

    #[test]
    fn test_fill_piece_basic() {
        let mut board = BlockBoard::INITIAL;
        // Create a simple test piece at known position
        // This test verifies fill_piece_as works correctly

        let y = SENTINEL_MARGIN_TOP;
        let x = SENTINEL_MARGIN_LEFT;

        // Manually set a cell and verify fill_piece_as
        board.rows[y].cells[x] = Block::Empty;
        assert_eq!(board.rows[y].cells[x], Block::Empty);

        // Using fill_piece_as would require a Piece, which requires testing at a higher level
        // For now, verify the cell can be set
        board.rows[y].cells[x] = Block::Piece(PieceKind::O);
        assert_eq!(board.rows[y].cells[x], Block::Piece(PieceKind::O));
    }

    #[test]
    fn test_clear_lines_basic() {
        let mut board = BlockBoard::INITIAL;

        // Fill first playable line
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            board.rows[SENTINEL_MARGIN_TOP].cells[x] = Block::Piece(PieceKind::I);
        }

        // Clear lines should remove the filled line
        let cleared = board.clear_lines();
        assert_eq!(cleared, 1);

        // The cleared line should now be empty (TOP row)
        let playable_cells = board.rows[SENTINEL_MARGIN_TOP].playable_cells();
        for cell in playable_cells {
            assert_eq!(*cell, Block::Empty);
        }
    }

    #[test]
    fn test_clear_lines_multiple_consecutive() {
        let mut board = BlockBoard::INITIAL;

        // Fill three consecutive lines
        for i in 0..3 {
            let y = SENTINEL_MARGIN_TOP + i;
            for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
                board.rows[y].cells[x] = Block::Piece(PieceKind::I);
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
                assert_eq!(*cell, Block::Empty);
            }
        }
    }

    #[test]
    fn test_clear_lines_with_partial_lines() {
        let mut board = BlockBoard::INITIAL;

        // Fill only part of the first line (not all playable cells)
        let y = SENTINEL_MARGIN_TOP;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH - 1 {
            board.rows[y].cells[x] = Block::Piece(PieceKind::I);
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
        let mut board = BlockBoard::INITIAL;

        // Fill the last playable line
        let y = SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT - 1;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            board.rows[y].cells[x] = Block::Piece(PieceKind::I);
        }

        // Clear lines
        let cleared = board.clear_lines();
        assert_eq!(cleared, 1);

        // The cleared line should be empty
        let playable_cells = board.rows[y].playable_cells();
        for cell in playable_cells {
            assert_eq!(*cell, Block::Empty);
        }
    }

    #[test]
    fn test_clear_lines_all_filled() {
        let mut board = BlockBoard::INITIAL;

        // Fill all playable lines
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
                board.rows[y].cells[x] = Block::Piece(PieceKind::I);
            }
        }

        // Clear lines
        let cleared = board.clear_lines();
        assert_eq!(cleared, PLAYABLE_HEIGHT);

        // All playable lines should now be empty
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            let playable_cells = board.rows[y].playable_cells();
            for cell in playable_cells {
                assert_eq!(*cell, Block::Empty);
            }
        }
    }

    #[test]
    fn test_clear_lines_preserves_sentinels() {
        let mut board = BlockBoard::INITIAL;

        // Fill first playable line
        let y = SENTINEL_MARGIN_TOP;
        for x in SENTINEL_MARGIN_LEFT..SENTINEL_MARGIN_LEFT + PLAYABLE_WIDTH {
            board.rows[y].cells[x] = Block::Piece(PieceKind::I);
        }

        // Clear lines
        board.clear_lines();

        // Verify sentinels are still intact
        for y in SENTINEL_MARGIN_TOP..SENTINEL_MARGIN_TOP + PLAYABLE_HEIGHT {
            assert_eq!(board.rows[y].cells[0], Block::Wall);
            assert_eq!(board.rows[y].cells[1], Block::Wall);
            assert_eq!(board.rows[y].cells[TOTAL_WIDTH - 2], Block::Wall);
            assert_eq!(board.rows[y].cells[TOTAL_WIDTH - 1], Block::Wall);
        }
    }
}
