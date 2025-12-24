use oxidris_engine::{BitBoard, GameState, Piece, PieceKind};
use std::{fmt, iter};

// All evaluation metrics are transformed into a [0.0, 1.0] score,
// where higher is always better.
//
// Normalization ranges are based on practical in-game observations
// (approximately the 95% percentile), not theoretical maxima.
// This preserves resolution and stabilizes GA optimization.

#[derive(Clone)]
pub(crate) struct MetricValues {
    pub covered_holes: f32,
    pub row_transitions: f32,
    pub column_transitions: f32,
    pub surface_roughness: f32,
    pub height_risk: f32,
    pub deep_wells: f32,
    pub sum_of_heights: f32,
    pub lines_cleared: f32,
    pub i_well_reward: f32,
}

impl fmt::Debug for MetricValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            covered_holes,
            row_transitions,
            column_transitions,
            surface_roughness,
            height_risk,
            deep_wells,
            sum_of_heights,
            lines_cleared,
            i_well_reward,
        } = *self;
        f.debug_struct("MetricValues")
            .field("covered_holes", &F32Fmt(covered_holes))
            .field("row_transitions", &F32Fmt(row_transitions))
            .field("column_transitions", &F32Fmt(column_transitions))
            .field("surface_roughness", &F32Fmt(surface_roughness))
            .field("height_risk", &F32Fmt(height_risk))
            .field("deep_wells", &F32Fmt(deep_wells))
            .field("sum_of_heights", &F32Fmt(sum_of_heights))
            .field("lines_cleared", &F32Fmt(lines_cleared))
            .field("i_well_reward", &F32Fmt(i_well_reward))
            .finish()
    }
}

struct F32Fmt(f32);
impl fmt::Debug for F32Fmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 0.123456789 -> 0.123_456_789
        let s = format!("{:?}", self.0);
        let (int, frac) = s.split_once('.').unwrap();
        write!(f, "{int}.")?;
        for (i, c) in frac.chars().enumerate() {
            if i > 0 && i % 3 == 0 {
                write!(f, "_")?;
            }
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

pub(crate) const METRIC_COUNT: usize = size_of::<MetricValues>() / size_of::<f32>();

impl MetricValues {
    pub(crate) const fn from_array(arr: [f32; METRIC_COUNT]) -> Self {
        Self {
            covered_holes: arr[0],
            row_transitions: arr[1],
            column_transitions: arr[2],
            surface_roughness: arr[3],
            height_risk: arr[4],
            deep_wells: arr[5],
            sum_of_heights: arr[6],
            lines_cleared: arr[7],
            i_well_reward: arr[8],
        }
    }

    pub(crate) const fn to_array(&self) -> [f32; METRIC_COUNT] {
        [
            self.covered_holes,
            self.row_transitions,
            self.column_transitions,
            self.surface_roughness,
            self.height_risk,
            self.deep_wells,
            self.sum_of_heights,
            self.lines_cleared,
            self.i_well_reward,
        ]
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Metrics(MetricValues);

impl Metrics {
    pub(crate) fn to_array(&self) -> [f32; METRIC_COUNT] {
        self.0.to_array()
    }

    pub(crate) fn measure(init: &GameState, game: &GameState, last_placement: Piece) -> Self {
        let line_clear_info = LineClearInfo::compute(init, game);
        let height_info = HeightInfo::compute(game.board());

        Self(MetricValues {
            covered_holes: height_info.covered_holes_score(),
            row_transitions: row_transitions_score(game.board()),
            column_transitions: column_transitions_score(game.board()),
            surface_roughness: height_info.surface_roughness_score(),
            height_risk: height_info.height_risk_score(),
            deep_wells: height_info.deep_wells_score(),
            sum_of_heights: height_info.sum_of_heights_score(),
            lines_cleared: line_clear_info.lines_cleared_score(),
            i_well_reward: height_info.i_well_reward(last_placement),
        })
    }
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

    fn lines_cleared_score(self) -> f32 {
        // Lines cleared represent forward progress and efficiency.
        // Weights strongly favor tetrises (4-line clears).
        //
        // Weighted score per N moves:
        //   mostly singles : ~0–N
        //   mixed clears   : ~2N–4N
        //   frequent tetrises : up to 6N
        //
        // sqrt() is applied to reduce variance and prevent domination
        // by rare high-reward events.
        const WEIGHT: [f32; 5] = [0.0, 0.0, 1.0, 2.0, 6.0];
        let raw = iter::zip(self.counter, WEIGHT)
            .map(|(count, weight)| f32::from(count) * weight)
            .sum::<f32>();
        let turns = f32::from(self.turns);
        let norm = normalize(raw.sqrt(), (turns * 6.0).sqrt());
        positive_metrics_score(norm)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct HeightInfo {
    heights: [u8; BitBoard::PLAYABLE_WIDTH],
    occupied: [u8; BitBoard::PLAYABLE_WIDTH],
    well_depths: [u8; BitBoard::PLAYABLE_WIDTH],
}

impl HeightInfo {
    pub(crate) fn compute(board: &BitBoard) -> Self {
        let mut heights = [0; BitBoard::PLAYABLE_WIDTH];
        let mut occupied = [0; BitBoard::PLAYABLE_WIDTH];
        for (i, x) in BitBoard::PLAYABLE_X_RANGE.enumerate() {
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

        let left = heights.into_iter().skip(1).chain(iter::once(u8::MAX));
        let right = iter::once(u8::MAX).chain(heights);
        let wells = iter::zip(heights, iter::zip(left, right)).map(|(h, (l, r))| {
            if h < l && h < r { u8::min(l, r) - h } else { 0 }
        });
        let mut well_depths = [0; BitBoard::PLAYABLE_WIDTH];
        for (i, depth) in wells.enumerate() {
            well_depths[i] = depth;
        }

        Self {
            heights,
            occupied,
            well_depths,
        }
    }

    pub(crate) fn covered_holes(&self) -> u16 {
        core::iter::zip(&self.heights, &self.occupied)
            .map(|(&h, &occ)| u16::from(h - occ))
            .sum()
    }

    pub(crate) fn covered_holes_score(&self) -> f32 {
        // Covered holes are empty cells with at least one block above them.
        // They are one of the strongest losing factors.
        //
        // Typical ranges (raw hole count):
        //   0–3   : very clean board
        //   4–7   : dangerous, recovery becomes difficult
        //   10+   : near-losing position
        //deep_
        // A power transform (holes^1.5) emphasizes early hole creation.
        // The normalization max (~60) corresponds to ~15 practical holes.
        let raw = f32::from(self.covered_holes()).powf(1.5);
        let norm = normalize(raw, 60.0);
        negative_metrics_score(norm)
    }

    fn sum_of_heights_score(&self) -> f32 {
        // Sum of column heights represents overall board pressure.
        // It correlates with reduced mobility and imminent top-out.
        //
        // Typical ranges (sum of heights, 10x20 board):
        //   40–60   : early game, very safe
        //   80–120  : mid game, manageable
        //   140+    : near top-out, highly dangerous
        //
        // max = 160 is chosen as a "95% practical limit",
        // not the theoretical maximum.
        let raw = self.heights.into_iter().map(f32::from).sum::<f32>();
        let norm = normalize(raw, 160.0);
        negative_metrics_score(norm)
    }

    fn surface_roughness_score(&self) -> f32 {
        // Surface roughness measures local curvature of the board surface
        // using second-order height differences.
        //
        // Unlike row transitions, this metric remains sensitive
        // when the overall stack is low.
        //
        // Typical ranges:
        //   0–5    : flat or well-shaped surface
        //   10–20  : normal mid-game roughness
        //   30+    : chaotic surface with high hole risk
        //
        // This metric complements row transitions rather than replacing it.
        let raw = self
            .heights
            .windows(3)
            .map(|w| {
                let left = i16::from(w[0]);
                let mid = i16::from(w[1]);
                let right = i16::from(w[2]);
                ((right - mid) - (mid - left)).unsigned_abs()
            })
            .sum::<u16>();
        let raw = f32::from(raw);
        let norm = normalize(raw, 40.0);
        negative_metrics_score(norm)
    }

    fn deep_wells_score(&self) -> f32 {
        // Deep wells detect excessively deep vertical gaps (width = 1).
        // Controlled shallow wells are intentionally ignored to allow
        // I-well (Tetris) strategies.
        //
        // Only wells deeper than 5 are considered dangerous.
        //
        // raw = sum of (well_depth^2) for dangerous wells
        // This aggressively penalizes over-committed vertical structures.
        //
        // Typical interpretation (10x20 board):
        //   raw ≈ 0      : no dangerous wells (safe or controlled I-wells)
        //   raw ≈ 10–20  : risky but potentially recoverable
        //   raw ≥ 50     : highly unstable, near-fatal structure
        //
        // This metric is NOT a positive reward.
        // It acts purely as a safety penalty using exponential decay,
        // while preserving freedom to build shallow I-wells.
        let raw = self
            .well_depths
            .into_iter()
            .map(|depth| {
                let depth = u16::from(depth);
                depth * depth
            })
            .sum::<u16>();
        let raw = f32::from(raw);
        let norm = normalize(raw, 50.0);
        // do not use negative_metrics_score here
        (-norm).exp()
    }

    pub(crate) fn max_height(&self) -> u8 {
        *self.heights.iter().max().unwrap()
    }

    fn height_risk_score(&self) -> f32 {
        // Height risk represents imminent top-out danger.
        // Heights below the safe threshold are intentionally ignored,
        // as moderate stacking is often necessary for line clears
        // and I-well construction.
        // A sharp exponential penalty is applied only near the ceiling
        // to model the irreversible nature of top-out.
        const SAFE_THRESHOLD: f32 = 0.7;

        let max_height = f32::from(self.max_height());
        #[expect(clippy::cast_precision_loss)]
        let board_height = BitBoard::PLAYABLE_HEIGHT as f32;
        let h = max_height / board_height;
        let raw = if h <= SAFE_THRESHOLD {
            0.0
        } else {
            (h - SAFE_THRESHOLD) / (1.0 - SAFE_THRESHOLD)
        };
        (-4.0 * raw).exp()
    }

    fn i_well_reward(&self, last_placement: Piece) -> f32 {
        let mut best_reward = 0.0;
        let mut best_depth = 0.0;
        for (i, depth) in self.well_depths.into_iter().enumerate() {
            if depth < 1 {
                continue;
            }
            let depth = f32::from(depth);
            let dist_to_edge = usize::min(i, BitBoard::PLAYABLE_WIDTH - 1 - i);
            #[expect(clippy::cast_precision_loss)]
            let dist = (dist_to_edge as f32) / (BitBoard::PLAYABLE_WIDTH as f32 / 2.0);

            let peak = 4.5;
            let sigma = 2.0;
            let depth_score = (-(depth - peak).powi(2) / (2.0 * sigma * sigma)).exp();

            let edge_bonus = (-dist).exp();

            let reward = depth_score * edge_bonus;
            if reward > best_reward {
                best_depth = depth;
                best_reward = reward;
            }
        }
        if best_depth >= 4.0 && last_placement.kind() == PieceKind::I {
            return 0.0;
        }
        best_reward
    }
}

fn row_transitions(board: &BitBoard) -> u16 {
    let mut transitions = 0;
    for row in board.playable_rows() {
        let mut cells = row.iter_playable_cells();
        let mut prev_occupied = cells.next().unwrap(); // left wall
        for occupied in cells {
            if occupied != prev_occupied {
                transitions += 1;
            }
            prev_occupied = occupied;
        }
    }
    transitions
}

fn row_transitions_score(board: &BitBoard) -> f32 {
    // Row transitions measure how fragmented each row is by counting
    // horizontal changes between occupied and empty cells.
    //
    // Only transitions *within* the playable area are counted.
    // Board walls are intentionally ignored to avoid artificial
    // incentives for stacking against the edges.
    //
    // This metric penalizes:
    //   - fragmented horizontal structures
    //   - narrow gaps and broken surfaces
    //   - layouts that are difficult to clear efficiently
    //
    // Typical ranges (10x20 board, wall-ignored):
    //   20–40   : very clean and flat structure
    //   60–90   : normal mid-game roughness
    //   120+    : highly fragmented, unstable board
    //
    // This is a negative metric; lower transition counts are better.
    let raw = f32::from(row_transitions(board));
    let norm = normalize(raw, 120.0);
    negative_metrics_score(norm)
}

fn column_transitions(board: &BitBoard) -> u16 {
    let mut transitions = 0;
    for x in BitBoard::PLAYABLE_X_RANGE {
        let mut prev_occupied = board.playable_row(0).is_cell_occupied(x); // top cell
        for y in 1..BitBoard::PLAYABLE_HEIGHT {
            let occupied = board.playable_row(y).is_cell_occupied(x);
            if occupied != prev_occupied {
                transitions += 1;
            }
            prev_occupied = occupied;
        }
        if !prev_occupied {
            transitions += 1; // bottom wall
        }
    }
    transitions
}

fn column_transitions_score(board: &BitBoard) -> f32 {
    // Column transitions count vertical occupancy changes per column,
    // treating covered holes as empty cells.
    //
    // This metric captures vertical fragmentation that is not always
    // visible from row transitions alone.
    //
    // Typical ranges (10x20 board):
    //   20–40   : clean vertical structure
    //   60–100  : normal mid-game fragmentation
    //   120+    : severe vertical instability
    //
    // Covered holes are intentionally treated as empty here,
    // since they are already penalized by a dedicated metric.
    let raw = f32::from(column_transitions(board));
    let norm = normalize(raw, 120.0);
    negative_metrics_score(norm)
}

fn normalize(value: f32, max: f32) -> f32 {
    (value / max).clamp(0.0, 1.0)
}

fn positive_metrics_score(norm: f32) -> f32 {
    norm
}

fn negative_metrics_score(norm: f32) -> f32 {
    1.0 - norm
}
