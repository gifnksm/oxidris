use crate::core::piece::Piece;
use RenderCell::{Empty as E, Wall as W};

use super::piece::PieceKind;

const PLAYABLE_WIDTH: usize = 10;
const PLAYABLE_HEIGHT: usize = 20;

const TOTAL_WIDTH: usize = PLAYABLE_WIDTH + (SENTINEL_MARGIN_LEFT + SENTINEL_MARGIN_RIGHT);
const TOTAL_HEIGHT: usize = PLAYABLE_HEIGHT + (SENTINEL_MARGIN_TOP + SENTINEL_MARGIN_BOTTOM);
pub(super) const TOTAL_MIN_X: usize = 0;
pub(super) const TOTAL_MAX_X: usize = TOTAL_WIDTH - 1;
pub(super) const TOTAL_MIN_Y: usize = 0;
pub(super) const TOTAL_MAX_Y: usize = TOTAL_HEIGHT - 1;
pub(super) const PIECE_SPAWN_X: usize = 5;
pub(super) const PIECE_SPAWN_Y: usize = 0;

const SENTINEL_MARGIN_TOP: usize = 2;
const SENTINEL_MARGIN_BOTTOM: usize = 2;
const SENTINEL_MARGIN_LEFT: usize = 2;
const SENTINEL_MARGIN_RIGHT: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub(crate) enum RenderCell {
    #[default]
    Empty,
    Wall,
    Ghost,
    Piece(PieceKind),
}

impl RenderCell {
    pub(crate) fn is_empty(self) -> bool {
        self == RenderCell::Empty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RenderRow {
    cells: [RenderCell; TOTAL_WIDTH],
}

impl RenderRow {
    // Board layout with 2-cell wall borders to enable proper piece movement.
    //
    // Why 2 cells instead of 1? All pieces use 4x4 grids for positioning and collision
    // detection. The key issue is that I-piece has 2 consecutive empty columns adjacent to
    // its cell, while other pieces have at most 1 empty column in any direction.
    //
    // This means I-piece needs more border space to move naturally near board edges.
    // Without sufficient borders, similar movement restriction problems would occur for
    // all pieces, but I-piece demonstrates the issue most clearly.
    //
    // The vertical I-piece occupies this 4x4 grid pattern:
    // (. = empty (E in code), I = I-piece cell, W = wall)
    //  [.  I  .  .]
    //  [.  I  .  .]
    //  [.  I  .  .]
    //  [.  I  .  .]
    //
    // Problem with 1-cell border (showing 4x4 grid constraints):
    //             0  1  2  3  4  5  6  7  8  9 10 11
    // Board:      W  .  .  .  .  .  .  .  .  .  .  W    ← 10-column playable area (column 1 to 10)
    // I-piece:   [W  I  .  .] .  .  .  .  .  .  .  W    ← Leftmost: I-piece cell at column 1
    // I-piece:    W  .  .  .  .  .  .  . [.  I  .  W]   ← Rightmost: I-piece cell at column 9, not 10
    //                                          ^^^
    //               The I-piece cannot reach right most columns due to 4x4 grid constraint
    //
    // Solution with 2-cell border:
    //             0  1  2  3  4  5  6  7  8  9 10 11 12 13
    // Board:      W  W  .  .  .  .  .  .  .  .  .  .  W  W   ← 10-column playable area (column 2 to 11)
    // I-piece:    W [W  I  .  .] .  .  .  .  .  .  .  W  W   ← Leftmost: I-piece cell at column 2
    // I-piece:    W  W  .  .  .  .  .  .  .  . [.  I  W  W]  ← Rightmost: I-piece cell at column 11
    //
    // This allows full movement range while maintaining 4x4 grid collision detection.
    const TOP: Self = RenderRow {
        cells: [W, W, E, E, E, E, E, E, E, E, E, E, W, W],
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
pub(crate) struct RenderBoard {
    rows: [RenderRow; TOTAL_HEIGHT],
}

impl RenderBoard {
    pub(crate) const PLAYABLE_WIDTH: usize = PLAYABLE_WIDTH;
    pub(crate) const PLAYABLE_HEIGHT: usize = PLAYABLE_HEIGHT;

    pub(crate) const INITIAL: Self = {
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

    pub(crate) fn playable_row(&self, y: usize) -> &[RenderCell; PLAYABLE_WIDTH] {
        self.rows[y + SENTINEL_MARGIN_TOP].playable_cells()
    }

    pub(crate) fn playable_rows(&self) -> impl Iterator<Item = &[RenderCell; PLAYABLE_WIDTH]> {
        self.rows[SENTINEL_MARGIN_TOP..][..PLAYABLE_HEIGHT]
            .iter()
            .map(RenderRow::playable_cells)
    }

    pub(crate) fn fill_piece(&mut self, piece: &Piece) {
        for (x, y) in piece.occupied_positions() {
            self.rows[y].cells[x] = RenderCell::Piece(piece.kind());
        }
    }

    pub(crate) fn fill_piece_as(&mut self, piece: &Piece, cell: RenderCell) {
        for (x, y) in piece.occupied_positions() {
            self.rows[y].cells[x] = cell;
        }
    }

    pub(crate) fn is_colliding(&self, piece: &Piece) -> bool {
        for (x, y) in piece.occupied_positions() {
            if !self.rows[y].cells[x].is_empty() {
                return true;
            }
        }
        false
    }

    pub(crate) fn clear_lines(&mut self) -> usize {
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
