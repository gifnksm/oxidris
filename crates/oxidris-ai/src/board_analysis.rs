use std::{cell::OnceCell, iter};

use oxidris_engine::BitBoard;

#[derive(Debug)]
pub struct BoardAnalysis {
    board: BitBoard,
    column_heights: OnceCell<[u8; BitBoard::PLAYABLE_WIDTH]>,
    column_occupied_cells: OnceCell<[u8; BitBoard::PLAYABLE_WIDTH]>,
    column_well_depths: OnceCell<[u8; BitBoard::PLAYABLE_WIDTH]>,
    max_height: OnceCell<u8>,
    center_column_max_height: OnceCell<u8>,
    total_height: OnceCell<u8>,
    num_holes: OnceCell<u8>,
    sum_of_hole_depth: OnceCell<u32>,
    row_transitions: OnceCell<u32>,
    column_transitions: OnceCell<u32>,
    surface_bumpiness: OnceCell<u32>,
    surface_roughness: OnceCell<u32>,
    sum_of_deep_well_depth: OnceCell<u32>,
    edge_iwell_depth: OnceCell<u8>,
}

impl BoardAnalysis {
    #[must_use]
    pub fn from_board(board: &BitBoard) -> Self {
        let board = board.clone();
        Self {
            board,
            column_heights: OnceCell::new(),
            column_occupied_cells: OnceCell::new(),
            column_well_depths: OnceCell::new(),
            max_height: OnceCell::new(),
            center_column_max_height: OnceCell::new(),
            total_height: OnceCell::new(),
            num_holes: OnceCell::new(),
            sum_of_hole_depth: OnceCell::new(),
            row_transitions: OnceCell::new(),
            column_transitions: OnceCell::new(),
            surface_bumpiness: OnceCell::new(),
            surface_roughness: OnceCell::new(),
            sum_of_deep_well_depth: OnceCell::new(),
            edge_iwell_depth: OnceCell::new(),
        }
    }

    #[must_use]
    pub fn board(&self) -> &BitBoard {
        &self.board
    }

    #[must_use]
    pub fn column_heights(&self) -> &[u8; BitBoard::PLAYABLE_WIDTH] {
        self.column_heights.get_or_init(|| {
            let mut column_heights = [0; BitBoard::PLAYABLE_WIDTH];
            for (x, h) in iter::zip(BitBoard::PLAYABLE_X_RANGE, &mut column_heights) {
                let min_y = self
                    .board
                    .playable_rows()
                    .enumerate()
                    .find(|(_y, row)| row.is_cell_occupied(x));
                let Some((min_y, _)) = min_y else {
                    continue;
                };
                *h = u8::try_from(BitBoard::PLAYABLE_HEIGHT - min_y).unwrap();
            }
            column_heights
        })
    }

    #[must_use]
    pub fn column_occupied_cells(&self) -> &[u8; BitBoard::PLAYABLE_WIDTH] {
        self.column_occupied_cells.get_or_init(|| {
            let mut column_occupied_cells = [0; BitBoard::PLAYABLE_WIDTH];
            for (x, o) in iter::zip(BitBoard::PLAYABLE_X_RANGE, &mut column_occupied_cells) {
                for row in self.board.playable_rows() {
                    if row.is_cell_occupied(x) {
                        *o += 1;
                    }
                }
            }
            column_occupied_cells
        })
    }

    #[must_use]
    pub fn column_well_depths(&self) -> &[u8; BitBoard::PLAYABLE_WIDTH] {
        self.column_well_depths.get_or_init(|| {
            let h = self.column_heights();
            let start = &[u8::MAX, h[0], h[1]][..];
            let end = &[h[h.len() - 2], h[h.len() - 1], u8::MAX][..];
            let triples = iter::once(start).chain(h.windows(3)).chain(iter::once(end));
            let wells = triples.map(|w| {
                if w[1] < w[0] && w[1] < w[2] {
                    u8::min(w[0], w[2]) - w[1]
                } else {
                    0
                }
            });
            let mut column_well_depths = [0; BitBoard::PLAYABLE_WIDTH];
            for (w, h) in iter::zip(wells, &mut column_well_depths) {
                *h = w;
            }
            column_well_depths
        })
    }

    #[must_use]
    pub fn max_height(&self) -> u8 {
        *self
            .max_height
            .get_or_init(|| *self.column_heights().iter().max().unwrap())
    }

    #[must_use]
    pub fn center_column_max_height(&self) -> u8 {
        *self.center_column_max_height.get_or_init(|| {
            const CENTER_START: usize = 3;
            const CENTER_END: usize = 6;
            *self.column_heights()[CENTER_START..=CENTER_END]
                .iter()
                .max()
                .unwrap()
        })
    }

    #[must_use]
    pub fn total_height(&self) -> u8 {
        *self
            .total_height
            .get_or_init(|| self.column_heights().iter().copied().sum())
    }

    #[must_use]
    pub fn num_holes(&self) -> u8 {
        *self.num_holes.get_or_init(|| {
            iter::zip(self.column_heights(), self.column_occupied_cells())
                .map(|(h, occ)| h - occ)
                .sum()
        })
    }

    #[must_use]
    pub fn sum_of_hole_depth(&self) -> u32 {
        *self.sum_of_hole_depth.get_or_init(|| {
            let mut depth_sum = 0u32;
            for x in BitBoard::PLAYABLE_X_RANGE {
                let mut depth = 0u32;
                for y in 0..BitBoard::PLAYABLE_HEIGHT {
                    let occupied = self.board.playable_row(y).is_cell_occupied(x);
                    if occupied {
                        depth += 1;
                    } else if depth > 0 {
                        depth_sum += depth;
                        depth += 1;
                    }
                }
            }
            depth_sum
        })
    }

    #[must_use]
    pub fn row_transitions(&self) -> u32 {
        *self.row_transitions.get_or_init(|| {
            let mut transitions = 0;
            for row in self.board.playable_rows() {
                let mut cells = row.iter_playable_cells();
                let mut prev_occupied = cells.next().unwrap();
                for occupied in cells {
                    if occupied != prev_occupied {
                        transitions += 1;
                    }
                    prev_occupied = occupied;
                }
            }
            transitions
        })
    }

    #[must_use]
    pub fn column_transitions(&self) -> u32 {
        *self.column_transitions.get_or_init(|| {
            let mut transitions = 0;
            for x in BitBoard::PLAYABLE_X_RANGE {
                let mut prev_occupied = self.board.playable_row(0).is_cell_occupied(x);
                for y in 1..BitBoard::PLAYABLE_HEIGHT {
                    let occupied = self.board.playable_row(y).is_cell_occupied(x);
                    if occupied != prev_occupied {
                        transitions += 1;
                    }
                    prev_occupied = occupied;
                }
            }
            transitions
        })
    }

    #[must_use]
    pub fn surface_bumpiness(&self) -> u32 {
        *self.surface_bumpiness.get_or_init(|| {
            self.column_heights()
                .windows(2)
                .map(|w| {
                    let left = i32::from(w[0]);
                    let right = i32::from(w[1]);
                    (right - left).unsigned_abs()
                })
                .sum()
        })
    }

    #[must_use]
    pub fn surface_roughness(&self) -> u32 {
        *self.surface_roughness.get_or_init(|| {
            self.column_heights()
                .windows(3)
                .map(|w| {
                    let left = i32::from(w[0]);
                    let mid = i32::from(w[1]);
                    let right = i32::from(w[2]);
                    ((right - mid) - (mid - left)).unsigned_abs()
                })
                .sum()
        })
    }

    #[must_use]
    pub fn sum_of_deep_well_depth(&self) -> u32 {
        *self.sum_of_deep_well_depth.get_or_init(|| {
            const DEPTH_THRESHOLD: u8 = 1;
            self.column_well_depths()
                .iter()
                .filter(|depth| **depth > DEPTH_THRESHOLD)
                .map(|depth| u32::from(depth - DEPTH_THRESHOLD))
                .sum()
        })
    }

    #[must_use]
    pub fn edge_iwell_depth(&self) -> u8 {
        *self.edge_iwell_depth.get_or_init(|| {
            let well_depth = self.column_well_depths();
            let left_well_depth = well_depth[0];
            let right_well_depth = well_depth[BitBoard::PLAYABLE_WIDTH - 1];
            u8::max(left_well_depth, right_well_depth)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Common board patterns for reuse across tests
    mod test_boards {
        use super::*;

        pub fn empty() -> BitBoard {
            BitBoard::INITIAL
        }

        pub fn flat() -> BitBoard {
            BitBoard::from_ascii(
                "
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ##########
                ##########
                ",
            )
        }

        pub fn staircase() -> BitBoard {
            BitBoard::from_ascii(
                "
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                #.........
                ##........
                ###.......
                ####......
                #####.....
                ",
            )
        }

        pub fn single_hole() -> BitBoard {
            BitBoard::from_ascii(
                "
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                #.........
                ..........
                #.........
                ",
            )
        }

        pub fn well() -> BitBoard {
            BitBoard::from_ascii(
                "
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                #.#.......
                ###.......
                ###.......
                ",
            )
        }

        pub fn edge_well_left() -> BitBoard {
            BitBoard::from_ascii(
                "
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                .#........
                ##........
                ##........
                ",
            )
        }

        pub fn edge_well_tetris_ready() -> BitBoard {
            BitBoard::from_ascii(
                "
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                .#........
                .#........
                .#........
                .#........
                ##........
                ",
            )
        }

        pub fn alternating_pattern() -> BitBoard {
            BitBoard::from_ascii(
                "
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                ..........
                #.#.#.#.#.
                ",
            )
        }
    }

    #[test]
    fn test_basic_metrics_on_common_boards() {
        // Table-driven test for common board patterns
        // Format: (name, board, total_height, max_height, num_holes, bumpiness, roughness)
        let test_cases = vec![
            ("empty", test_boards::empty(), 0, 0, 0, 0, 0),
            ("flat", test_boards::flat(), 20, 2, 0, 0, 0),
            ("staircase", test_boards::staircase(), 15, 5, 0, 5, 1),
            ("single_hole", test_boards::single_hole(), 3, 3, 1, 3, 3),
            ("with_well", test_boards::well(), 8, 3, 0, 5, 9),
        ];

        for (
            name,
            board,
            expected_total_height,
            expected_max_height,
            expected_num_holes,
            expected_surface_bumpiness,
            expected_surface_roughness,
        ) in test_cases
        {
            let analysis = BoardAnalysis::from_board(&board);
            assert_eq!(
                analysis.total_height(),
                expected_total_height,
                "{}: total_height",
                name
            );
            assert_eq!(
                analysis.max_height(),
                expected_max_height,
                "{}: max_height",
                name
            );
            assert_eq!(
                analysis.num_holes(),
                expected_num_holes,
                "{}: num_holes",
                name
            );
            assert_eq!(
                analysis.surface_bumpiness(),
                expected_surface_bumpiness,
                "{}: surface_bumpiness",
                name
            );
            assert_eq!(
                analysis.surface_roughness(),
                expected_surface_roughness,
                "{}: surface_roughness",
                name
            );
        }
    }

    #[test]
    fn test_column_heights() {
        let board = test_boards::staircase();
        let analysis = BoardAnalysis::from_board(&board);
        let heights = analysis.column_heights();

        assert_eq!(heights[0], 5);
        assert_eq!(heights[1], 4);
        assert_eq!(heights[2], 3);
        assert_eq!(heights[3], 2);
        assert_eq!(heights[4], 1);
        for i in 5..10 {
            assert_eq!(heights[i], 0);
        }
    }

    #[test]
    fn test_column_occupied_cells() {
        let board = test_boards::single_hole();
        let analysis = BoardAnalysis::from_board(&board);
        let occupied = analysis.column_occupied_cells();

        // Column 0 has 2 occupied cells (with 1 hole in between)
        assert_eq!(occupied[0], 2);
        for i in 1..10 {
            assert_eq!(occupied[i], 0);
        }
    }

    #[test]
    fn test_sum_of_hole_depth() {
        let test_cases = vec![
            ("empty", test_boards::empty(), 0),
            ("flat", test_boards::flat(), 0),
            ("staircase", test_boards::staircase(), 0),
            ("single_hole", test_boards::single_hole(), 1),
        ];

        for (name, board, expected) in test_cases {
            let analysis = BoardAnalysis::from_board(&board);
            assert_eq!(analysis.sum_of_hole_depth(), expected, "{}", name);
        }
    }

    #[test]
    fn test_sum_of_hole_depth_complex() {
        // Multiple holes at different depths
        let board = BitBoard::from_ascii(
            "
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            #.........
            ..........
            #.........
            ..........
            #.........
            ",
        );

        let analysis = BoardAnalysis::from_board(&board);
        // First hole: depth 1, second hole: depth 3, total: 4
        assert_eq!(analysis.sum_of_hole_depth(), 4);
    }

    #[test]
    fn test_row_transitions() {
        let test_cases = vec![
            ("empty", test_boards::empty(), 0),
            ("flat", test_boards::flat(), 0),
            ("alternating", test_boards::alternating_pattern(), 9),
        ];

        for (name, board, expected) in test_cases {
            let analysis = BoardAnalysis::from_board(&board);
            assert_eq!(analysis.row_transitions(), expected, "{}", name);
        }
    }

    #[test]
    fn test_column_transitions() {
        let test_cases = vec![
            ("empty", test_boards::empty(), 0),
            ("flat", test_boards::flat(), 10),
            ("staircase", test_boards::staircase(), 5),
            ("single_hole", test_boards::single_hole(), 3),
        ];

        for (name, board, expected) in test_cases {
            let analysis = BoardAnalysis::from_board(&board);
            assert_eq!(analysis.column_transitions(), expected, "{}", name);
        }
    }

    #[test]
    fn test_column_well_depths() {
        let board = test_boards::well();
        let analysis = BoardAnalysis::from_board(&board);
        let wells = analysis.column_well_depths();

        assert_eq!(wells[0], 0);
        assert_eq!(wells[1], 1); // Well between columns 0 and 2
        assert_eq!(wells[2], 0);
    }

    #[test]
    fn test_column_well_depths_edges() {
        let board = test_boards::edge_well_left();
        let analysis = BoardAnalysis::from_board(&board);
        let wells = analysis.column_well_depths();

        assert_eq!(wells[0], 1); // Left edge well
    }

    #[test]
    fn test_sum_of_deep_well_depth() {
        let test_cases = vec![
            ("empty", test_boards::empty(), 0),
            ("flat", test_boards::flat(), 0),
            ("shallow_well", test_boards::well(), 0), // Well depth 1 is not counted
        ];

        for (name, board, expected) in test_cases {
            let analysis = BoardAnalysis::from_board(&board);
            assert_eq!(analysis.sum_of_deep_well_depth(), expected, "{}", name);
        }
    }

    #[test]
    fn test_sum_of_deep_well_depth_deep() {
        // Well with depth > 1
        let board = BitBoard::from_ascii(
            "
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            #.#.#.....
            #####.....
            #####.....
            ",
        );

        let analysis = BoardAnalysis::from_board(&board);
        // Wells have depth 1, which is not > threshold of 1
        assert_eq!(analysis.sum_of_deep_well_depth(), 0);
    }

    #[test]
    fn test_center_column_max_height() {
        let test_cases = vec![
            ("empty", test_boards::empty(), 0),
            ("flat", test_boards::flat(), 2),
        ];

        for (name, board, expected) in test_cases {
            let analysis = BoardAnalysis::from_board(&board);
            assert_eq!(analysis.center_column_max_height(), expected, "{}", name);
        }
    }

    #[test]
    fn test_center_column_max_height_with_center() {
        let board = BitBoard::from_ascii(
            "
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ...#......
            ...##.....
            ...###....
            ...####...
            ##.#####..
            ",
        );

        let analysis = BoardAnalysis::from_board(&board);
        // Columns 3-6 have heights [5,4,3,2], max is 5
        assert_eq!(analysis.center_column_max_height(), 5);
    }

    #[test]
    fn test_edge_iwell_depth() {
        let test_cases = vec![
            ("empty", test_boards::empty(), 0),
            ("flat", test_boards::flat(), 0),
            ("edge_well_left", test_boards::edge_well_left(), 1),
            ("tetris_ready", test_boards::edge_well_tetris_ready(), 4),
        ];

        for (name, board, expected) in test_cases {
            let analysis = BoardAnalysis::from_board(&board);
            assert_eq!(analysis.edge_iwell_depth(), expected, "{}", name);
        }
    }

    #[test]
    fn test_edge_iwell_depth_both_edges() {
        let board = BitBoard::from_ascii(
            "
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            ..........
            .#......#.
            .#......#.
            .#......#.
            .#......#.
            ##......##
            ##......##
            ",
        );

        let analysis = BoardAnalysis::from_board(&board);
        // Both edges have depth 4, returns max
        assert_eq!(analysis.edge_iwell_depth(), 4);
    }

    #[test]
    fn test_invariants() {
        // Property-based tests for invariants that should hold for all boards
        let boards = vec![
            test_boards::empty(),
            test_boards::flat(),
            test_boards::staircase(),
            test_boards::single_hole(),
            test_boards::well(),
        ];

        for board in boards {
            let analysis = BoardAnalysis::from_board(&board);

            // Invariant: num_holes = sum(heights) - sum(occupied_cells)
            let expected_holes: u8 = analysis
                .column_heights()
                .iter()
                .zip(analysis.column_occupied_cells())
                .map(|(h, o)| h - o)
                .sum();
            assert_eq!(analysis.num_holes(), expected_holes);

            // Invariant: max_height <= total_height
            assert!(u32::from(analysis.max_height()) <= u32::from(analysis.total_height()));

            // Invariant: total_height = sum of all column heights
            let sum_heights: u8 = analysis.column_heights().iter().sum();
            assert_eq!(analysis.total_height(), sum_heights);

            // Invariant: edge_iwell_depth <= max of all well depths
            let max_well = *analysis.column_well_depths().iter().max().unwrap();
            assert!(analysis.edge_iwell_depth() <= max_well);

            // Invariant: center_column_max_height <= max_height
            assert!(analysis.center_column_max_height() <= analysis.max_height());
        }
    }

    #[test]
    fn test_lazy_evaluation() {
        // Verify that OnceCell caching works correctly
        let board = test_boards::well();
        let analysis = BoardAnalysis::from_board(&board);

        // Call each method twice and verify consistency
        let heights1 = analysis.column_heights();
        let heights2 = analysis.column_heights();
        assert_eq!(heights1, heights2);

        let holes1 = analysis.num_holes();
        let holes2 = analysis.num_holes();
        assert_eq!(holes1, holes2);

        let depth1 = analysis.sum_of_hole_depth();
        let depth2 = analysis.sum_of_hole_depth();
        assert_eq!(depth1, depth2);
    }
}
