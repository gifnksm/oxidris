use crate::{
    core::bit_board::{BitBoard, SENTINEL_MARGIN_LEFT},
    engine::state::GameState,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Metric {
    LinesCleared = 0,
    HeightMax = 1,
    HeightDiff = 2,
    DeadSpace = 3,
}

pub(crate) const METRIC_COUNT: usize = 4;

pub(crate) fn measure(init: &GameState, game: &GameState) -> [f64; METRIC_COUNT] {
    let line_clear_info = LineClearInfo::compute(init, game);
    let height_info = HeightInfo::compute(game.board());

    let mut out = [0.0; METRIC_COUNT];

    out[Metric::LinesCleared as usize] = line_clear_info.normalized_lines_cleared();
    out[Metric::HeightMax as usize] = 1.0 - height_info.normalized_max_height();
    out[Metric::HeightDiff as usize] = 1.0 - height_info.normalized_height_diff();
    out[Metric::DeadSpace as usize] = 1.0 - height_info.normalized_dead_space();

    out
}

#[derive(Debug, Clone, Copy)]
struct LineClearInfo {
    turns: u8,
    counter: [u8; 5],
}

impl LineClearInfo {
    fn compute(init: &GameState, game: &GameState) -> Self {
        let turns = u8::try_from(game.completed_pieces() - init.completed_pieces()).unwrap();
        let counter = core::array::from_fn(|i| {
            u8::try_from(game.line_cleared_counter()[i] - init.line_cleared_counter()[i]).unwrap()
        });
        Self { turns, counter }
    }

    fn normalized_lines_cleared(self) -> f64 {
        const WEIGHT: [u8; 5] = [0, 0, 1, 2, 6];
        let min = 0;
        let max = self.turns * WEIGHT[4];
        let score = core::iter::zip(self.counter, WEIGHT)
            .map(|(count, weight)| count * weight)
            .sum::<u8>();
        normalize(score, min, max).sqrt()
    }
}

#[derive(Debug, Clone, Copy)]
struct HeightInfo {
    heights: [u8; BitBoard::PLAYABLE_WIDTH],
    occupied: [u8; BitBoard::PLAYABLE_WIDTH],
}

impl HeightInfo {
    fn compute(board: &BitBoard) -> Self {
        let mut heights = [0; BitBoard::PLAYABLE_WIDTH];
        let mut occupied = [0; BitBoard::PLAYABLE_WIDTH];
        for i in 0..BitBoard::PLAYABLE_WIDTH {
            let x = SENTINEL_MARGIN_LEFT + i;
            let min_y = board
                .playable_rows()
                .enumerate()
                .find(|(_y, row)| row.is_cell_occupied(x));
            let Some((min_y, _)) = min_y else {
                continue;
            };
            heights[i] = u8::try_from(BitBoard::PLAYABLE_HEIGHT - min_y).unwrap();
            occupied[i] = 1;
            for y in min_y + 1..BitBoard::PLAYABLE_HEIGHT {
                let row = board.playable_row(y);
                if row.is_cell_occupied(x) {
                    occupied[i] += 1;
                }
            }
        }
        Self { heights, occupied }
    }

    fn normalized_max_height(&self) -> f64 {
        const MIN: u8 = 0;
        #[allow(clippy::cast_possible_truncation)]
        const MAX: u8 = BitBoard::PLAYABLE_HEIGHT as u8;
        let height = *self.heights.iter().max().unwrap();
        normalize(height, MIN, MAX).powi(2)
    }

    fn normalized_height_diff(&self) -> f64 {
        const MIN: u8 = 0;
        #[allow(clippy::cast_possible_truncation)]
        const MAX: u8 = BitBoard::PLAYABLE_HEIGHT as u8 * BitBoard::PLAYABLE_WIDTH as u8;
        let diff = self
            .heights
            .iter()
            .zip(&self.heights[1..])
            .map(|(&a, &b)| a.abs_diff(b))
            .sum::<u8>();
        normalize(diff, MIN, MAX).powi(2)
    }

    fn normalized_dead_space(&self) -> f64 {
        const MIN: u8 = 0;
        #[allow(clippy::cast_possible_truncation)]
        const MAX: u8 = BitBoard::PLAYABLE_HEIGHT as u8 * BitBoard::PLAYABLE_WIDTH as u8;
        let dead_space = core::iter::zip(&self.heights, &self.occupied)
            .map(|(&h, &occ)| h - occ)
            .sum::<u8>();
        normalize(dead_space, MIN, MAX).sqrt()
    }
}

#[inline]
fn normalize(value: impl Into<f64>, min: impl Into<f64>, max: impl Into<f64>) -> f64 {
    let min = min.into();
    let max = max.into();
    let value = value.into().clamp(min, max);
    (value - min) / (max - min)
}
