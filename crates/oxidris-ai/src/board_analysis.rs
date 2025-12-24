use std::iter;

use oxidris_engine::{BitBoard, Piece};

pub struct BoardAnalysis {
    pub board: BitBoard,
    pub placement: Piece,
    pub cleared_lines: usize,
    pub column_heights: [u8; BitBoard::PLAYABLE_WIDTH],
    pub column_occupied_cells: [u8; BitBoard::PLAYABLE_WIDTH],
    pub column_well_depths: [u8; BitBoard::PLAYABLE_WIDTH],
}

impl BoardAnalysis {
    #[must_use]
    pub fn from_board(board: &BitBoard, placement: Piece) -> Self {
        let mut board = board.clone();
        board.fill_piece(placement);
        let cleared_lines = board.clear_lines();

        let mut column_heights = [0; BitBoard::PLAYABLE_WIDTH];
        let mut column_occupied_cells = [0; BitBoard::PLAYABLE_WIDTH];
        for (i, x) in BitBoard::PLAYABLE_X_RANGE.enumerate() {
            let min_y = board
                .playable_rows()
                .enumerate()
                .find(|(_y, row)| row.is_cell_occupied(x));
            let Some((min_y, _)) = min_y else {
                continue;
            };
            column_heights[i] = u8::try_from(BitBoard::PLAYABLE_HEIGHT - min_y).unwrap();
            column_occupied_cells[i] = 1;
            for y in min_y + 1..BitBoard::PLAYABLE_HEIGHT {
                let row = board.playable_row(y);
                if row.is_cell_occupied(x) {
                    column_occupied_cells[i] += 1;
                }
            }
        }

        let left = column_heights
            .into_iter()
            .skip(1)
            .chain(iter::once(u8::MAX));
        let right = iter::once(u8::MAX).chain(column_heights);
        let wells = iter::zip(column_heights, iter::zip(left, right)).map(|(h, (l, r))| {
            if h < l && h < r { u8::min(l, r) - h } else { 0 }
        });
        let mut column_well_depths = [0; BitBoard::PLAYABLE_WIDTH];
        for (i, depth) in wells.enumerate() {
            column_well_depths[i] = depth;
        }

        Self {
            board,
            placement,
            cleared_lines,
            column_heights,
            column_occupied_cells,
            column_well_depths,
        }
    }
}
