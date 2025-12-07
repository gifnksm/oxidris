use crate::core::{block::BlockKind, piece::Piece};
use BlockKind::{Empty as E, Wall as W};

const BOARD_WIDTH: usize = 12 + 2;
const BOARD_HEIGHT: usize = 22 + 1;
pub(super) const MIN_X: usize = 0;
pub(super) const MAX_X: usize = BOARD_WIDTH - 1;
pub(super) const MIN_Y: usize = 0;
pub(super) const MAX_Y: usize = BOARD_HEIGHT - 1;
pub(super) const INIT_PIECE_X: usize = 5;
pub(super) const INIT_PIECE_Y: usize = 0;

const BLOCKS_MARGIN_TOP: usize = 1;
const BLOCKS_MARGIN_BOTTOM: usize = 2;
const BLOCKS_MARGIN_LEFT: usize = 2;
const BLOCKS_MARGIN_RIGHT: usize = 2;
const BLOCKS_WIDTH: usize = BOARD_WIDTH - (BLOCKS_MARGIN_LEFT + BLOCKS_MARGIN_RIGHT);
const BLOCKS_HEIGHT: usize = BOARD_HEIGHT - (BLOCKS_MARGIN_TOP + BLOCKS_MARGIN_BOTTOM);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoardRow {
    cells: [BlockKind; BOARD_WIDTH],
}

impl BoardRow {
    // Board layout with 2-cell wall borders to enable proper piece movement.
    //
    // Why 2 cells instead of 1? All pieces use 4x4 grids for positioning and collision
    // detection. The key issue is that I-piece has 2 consecutive empty columns adjacent to
    // its block, while other pieces have at most 1 empty column in any direction.
    //
    // This means I-piece needs more border space to move naturally near board edges.
    // Without sufficient borders, similar movement restriction problems would occur for
    // all pieces, but I-piece demonstrates the issue most clearly.
    //
    // The vertical I-piece occupies this 4x4 grid pattern:
    // (. = empty (E in code), I = I-piece block, W = wall)
    //  [.  I  .  .]
    //  [.  I  .  .]
    //  [.  I  .  .]
    //  [.  I  .  .]
    //
    // Problem with 1-cell border (showing 4x4 grid constraints):
    //             0  1  2  3  4  5  6  7  8  9 10 11
    // Board:      W  .  .  .  .  .  .  .  .  .  .  W    ← 10-column playable area (column 1 to 10)
    // I-piece:   [W  I  .  .] .  .  .  .  .  .  .  W    ← Leftmost: I-block at column 1
    // I-piece:    W  .  .  .  .  .  .  . [.  I  .  W]   ← Rightmost: I-block at column 9, not 10
    //                                          ^^^
    //               The I-piece cannot reach right most columns due to 4x4 grid constraint
    //
    // Solution with 2-cell border:
    //             0  1  2  3  4  5  6  7  8  9 10 11 12 13
    // Board:      W  W  .  .  .  .  .  .  .  .  .  .  W  W   ← 10-column playable area (column 2 to 11)
    // I-piece:    W [W  I  .  .] .  .  .  .  .  .  .  W  W   ← Leftmost: I-block at column 2
    // I-piece:    W  W  .  .  .  .  .  .  .  . [.  I  W  W]  ← Rightmost: I-block at column 11
    //
    // This allows full movement range while maintaining 4x4 grid collision detection.
    const TOP: Self = BoardRow {
        cells: [W, W, E, E, E, E, E, E, E, E, E, E, W, W],
    };
    const BOTTOM: Self = BoardRow {
        cells: [W; BOARD_WIDTH],
    };

    fn blocks(&self) -> &[BlockKind; BLOCKS_WIDTH] {
        self.cells[BLOCKS_MARGIN_LEFT..][..BLOCKS_WIDTH]
            .try_into()
            .unwrap()
    }

    fn is_filled(&self) -> bool {
        self.blocks().iter().all(|b| !b.is_empty())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Board {
    rows: [BoardRow; BOARD_HEIGHT],
}

impl Board {
    pub(crate) const BLOCKS_WIDTH: usize = BLOCKS_WIDTH;
    pub(crate) const BLOCKS_HEIGHT: usize = BLOCKS_HEIGHT;

    pub(crate) const INITIAL: Self = {
        Self {
            rows: [
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::TOP,
                BoardRow::BOTTOM,
                BoardRow::BOTTOM,
            ],
        }
    };

    pub(crate) fn block_row(&self, y: usize) -> &[BlockKind; BLOCKS_WIDTH] {
        self.rows[y + 1].blocks()
    }

    pub(crate) fn block_rows(&self) -> impl Iterator<Item = &[BlockKind; BLOCKS_WIDTH]> {
        self.rows[BLOCKS_MARGIN_TOP..][..BLOCKS_HEIGHT]
            .iter()
            .map(BoardRow::blocks)
    }

    pub(crate) fn fill_piece(&mut self, piece: &Piece) {
        let pos = piece.position();
        for (y, piece_row) in piece.shape().iter().enumerate() {
            for (x, block) in piece_row.iter().enumerate() {
                if !block.is_empty() {
                    self.rows[y + pos.y()].cells[x + pos.x()] = *block;
                }
            }
        }
    }

    pub(crate) fn fill_piece_as(&mut self, piece: &Piece, kind: BlockKind) {
        let pos = piece.position();
        for (y, piece_row) in piece.shape().iter().enumerate() {
            for (x, block) in piece_row.iter().enumerate() {
                if !block.is_empty() {
                    self.rows[y + pos.y()].cells[x + pos.x()] = kind;
                }
            }
        }
    }

    pub(crate) fn is_colliding(&self, piece: &Piece) -> bool {
        let pos = piece.position();
        for (dy, piece_row) in piece.shape().iter().enumerate() {
            let y = dy + pos.y();
            for (dx, block) in piece_row.iter().enumerate() {
                let x = dx + pos.x();
                if !block.is_empty() && !self.rows[y].cells[x].is_empty() {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn clear_lines(&mut self) -> usize {
        let block_rows = &mut self.rows[BLOCKS_MARGIN_TOP..][..BLOCKS_HEIGHT];
        let mut count = 0;
        for y in (0..BLOCKS_HEIGHT).rev() {
            if block_rows[y].is_filled() {
                count += 1;
                continue;
            }
            if count > 0 {
                block_rows[y + count] = block_rows[y];
            }
        }
        block_rows[..count].fill(BoardRow::TOP);
        count
    }
}
