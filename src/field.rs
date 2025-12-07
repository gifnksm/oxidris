use crate::{block::BlockKind, mino::Mino};
use BlockKind::{Empty as E, Wall as W};

const FIELD_WIDTH: usize = 12 + 2;
const FIELD_HEIGHT: usize = 22 + 1;
pub(crate) const MIN_X: usize = 0;
pub(crate) const MAX_X: usize = FIELD_WIDTH - 1;
pub(crate) const MIN_Y: usize = 0;
pub(crate) const MAX_Y: usize = FIELD_HEIGHT - 1;
pub(crate) const INIT_MINO_X: usize = 5;
pub(crate) const INIT_MINO_Y: usize = 0;

const BLOCKS_MARGIN_TOP: usize = 1;
const BLOCKS_MARGIN_BOTTOM: usize = 2;
const BLOCKS_MARGIN_LEFT: usize = 2;
const BLOCKS_MARGIN_RIGHT: usize = 2;
const BLOCKS_WIDTH: usize = FIELD_WIDTH - (BLOCKS_MARGIN_LEFT + BLOCKS_MARGIN_RIGHT);
const BLOCKS_HEIGHT: usize = FIELD_HEIGHT - (BLOCKS_MARGIN_TOP + BLOCKS_MARGIN_BOTTOM);

#[derive(Debug, Clone)]
pub(crate) struct Field {
    rows: [[BlockKind; FIELD_WIDTH]; FIELD_HEIGHT],
}

// Field layout with 2-cell wall borders to enable proper tetramino movement.
//
// Why 2 cells instead of 1? All tetraminos use 4x4 grids for positioning and collision
// detection. The key issue is that I-mino has 2 consecutive empty columns adjacent to
// its block, while other tetraminos have at most 1 empty column in any direction.
//
// This means I-mino needs more border space to move naturally near field edges.
// Without sufficient borders, similar movement restriction problems would occur for
// all tetraminos, but I-mino demonstrates the issue most clearly.
//
// The vertical I-mino occupies this 4x4 grid pattern:
// (. = empty (E in code), I = I-mino block, W = wall)
//  [.  I  .  .]
//  [.  I  .  .]
//  [.  I  .  .]
//  [.  I  .  .]
//
// Problem with 1-cell border (showing 4x4 grid constraints):
//             0  1  2  3  4  5  6  7  8  9 10 11
// Field:      W  .  .  .  .  .  .  .  .  .  .  W    ← 10-column playable area (column 1 to 10)
// I-mino:    [W  I  .  .] .  .  .  .  .  .  .  W    ← Leftmost: I-block at column 1
// I-mino:     W  .  .  .  .  .  .  . [.  I  .  W]   ← Rightmost: I-block at column 9, not 10
//                                          ^^^
//               The I-mino cannot reach right most columns due to 4x4 grid constraint
//
// Solution with 2-cell border:
//             0  1  2  3  4  5  6  7  8  9 10 11 12 13
// Field:      W  W  .  .  .  .  .  .  .  .  .  .  W  W   ← 10-column playable area (column 2 to 11)
// I-mino:     W [W  I  .  .] .  .  .  .  .  .  .  W  W   ← Leftmost: I-block at column 2
// I-mino:     W  W  .  .  .  .  .  .  .  . [.  I  W  W]  ← Rightmost: I-block at column 11
//
// This allows full movement range while maintaining 4x4 grid collision detection.
const TOP_ROW: [BlockKind; FIELD_WIDTH] = [W, W, E, E, E, E, E, E, E, E, E, E, W, W];
const BOTTOM_ROW: [BlockKind; FIELD_WIDTH] = [W, W, W, W, W, W, W, W, W, W, W, W, W, W];

impl Field {
    pub(crate) const BLOCKS_WIDTH: usize = BLOCKS_WIDTH;
    pub(crate) const BLOCKS_HEIGHT: usize = BLOCKS_HEIGHT;

    pub(crate) const INITIAL: Self = {
        Self {
            rows: [
                TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW,
                TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW, TOP_ROW,
                TOP_ROW, TOP_ROW, TOP_ROW, BOTTOM_ROW, BOTTOM_ROW,
            ],
        }
    };

    pub(crate) fn block_row(&self, y: usize) -> &[BlockKind] {
        &self.rows[y + 1][BLOCKS_MARGIN_LEFT..][..BLOCKS_WIDTH]
    }

    pub(crate) fn block_rows(&self) -> impl Iterator<Item = &[BlockKind]> {
        self.rows[BLOCKS_MARGIN_TOP..][..BLOCKS_HEIGHT]
            .iter()
            .map(|row| &row[BLOCKS_MARGIN_LEFT..][..BLOCKS_WIDTH])
    }

    pub(crate) fn fill_mino(&mut self, mino: &Mino) {
        let pos = mino.position();
        for (y, mino_row) in mino.shape().iter().enumerate() {
            for (x, block) in mino_row.iter().enumerate() {
                if !block.is_empty() {
                    self.rows[y + pos.y()][x + pos.x()] = *block;
                }
            }
        }
    }

    pub(crate) fn fill_mino_as(&mut self, mino: &Mino, kind: BlockKind) {
        let pos = mino.position();
        for (y, mino_row) in mino.shape().iter().enumerate() {
            for (x, block) in mino_row.iter().enumerate() {
                if !block.is_empty() {
                    self.rows[y + pos.y()][x + pos.x()] = kind;
                }
            }
        }
    }

    pub(crate) fn is_colliding(&self, mino: &Mino) -> bool {
        let pos = mino.position();
        for (dy, mino_row) in mino.shape().iter().enumerate() {
            let y = dy + pos.y();
            if y >= FIELD_HEIGHT {
                continue;
            }
            for (dx, block) in mino_row.iter().enumerate() {
                let x = dx + pos.x();
                if x >= FIELD_WIDTH {
                    continue;
                }
                if !block.is_empty() && !self.rows[y][x].is_empty() {
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
            let can_erase = block_rows[y][BLOCKS_MARGIN_LEFT..][..BLOCKS_WIDTH]
                .iter()
                .all(|&v| !v.is_empty());
            if can_erase {
                count += 1;
                continue;
            }
            if count > 0 {
                block_rows[y + count] = block_rows[y];
            }
        }
        block_rows[..count].fill(TOP_ROW);
        count
    }
}
