/// Score values for line clears.
///
/// Index corresponds to number of lines cleared simultaneously:
/// - 0 lines: 0 points
/// - 1 line: 100 points
/// - 2 lines: 300 points
/// - 3 lines: 500 points
/// - 4 lines: 800 points
const SCORE_TABLE: [usize; 5] = [0, 100, 300, 500, 800];

/// Game statistics tracking score, lines cleared, and piece count.
///
/// Tracks various metrics during a game session:
///
/// - **Score**: Points earned from line clears
/// - **Level**: Derived from total lines cleared (1 level per 10 lines)
/// - **Completed pieces**: Total number of pieces locked
/// - **Line clear distribution**: Count of single, double, triple, quad line clears
///
/// # Scoring
///
/// Scoring is simplified compared to standard Tetris:
/// - No combo bonuses
/// - No back-to-back bonuses
/// - No T-spin scoring
///
/// See [Engine Implementation](../../../docs/architecture/engine/README.md#scoring) for details.
///
/// # Example
///
/// ```
/// use oxidris_engine::GameStats;
///
/// let mut stats = GameStats::new();
/// stats.complete_piece_drop(4); // Tetris (4 lines)
///
/// assert_eq!(stats.score(), 800);
/// assert_eq!(stats.total_cleared_lines(), 4);
/// assert_eq!(stats.line_cleared_counter()[4], 1);
/// ```
#[derive(Debug, Clone)]
pub struct GameStats {
    score: usize,
    completed_pieces: usize,
    total_cleared_lines: usize,
    line_cleared_counter: [usize; 5],
}

impl Default for GameStats {
    fn default() -> Self {
        Self::new()
    }
}

impl GameStats {
    /// Creates a new game statistics tracker with all counters at zero.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            score: 0,
            completed_pieces: 0,
            total_cleared_lines: 0,
            line_cleared_counter: [0; 5],
        }
    }

    /// Returns the current score (sum of all line clear points).
    #[must_use]
    pub const fn score(&self) -> usize {
        self.score
    }

    /// Returns the current level based on total lines cleared.
    ///
    /// Level increases by 1 for every 10 lines cleared (integer division).
    #[must_use]
    pub fn level(&self) -> usize {
        self.total_cleared_lines / 10
    }

    /// Returns the total number of pieces that have been locked into place.
    #[must_use]
    pub const fn completed_pieces(&self) -> usize {
        self.completed_pieces
    }

    /// Returns the total number of lines cleared across all line clears.
    #[must_use]
    pub const fn total_cleared_lines(&self) -> usize {
        self.total_cleared_lines
    }

    /// Returns a histogram of line clears by count.
    ///
    /// Array indices represent:
    /// - `[0]`: Number of drops with 0 lines cleared
    /// - `[1]`: Number of singles (1 line)
    /// - `[2]`: Number of doubles (2 lines)
    /// - `[3]`: Number of triples (3 lines)
    /// - `[4]`: Number of tetrises (4 lines)
    #[must_use]
    pub const fn line_cleared_counter(&self) -> &[usize; 5] {
        &self.line_cleared_counter
    }

    /// Updates statistics after a piece drop.
    ///
    /// This should be called each time a piece is locked into place.
    ///
    /// # Arguments
    ///
    /// * `cleared_lines` - Number of lines cleared (0-4)
    pub const fn complete_piece_drop(&mut self, cleared_lines: usize) {
        self.completed_pieces += 1;
        self.total_cleared_lines += cleared_lines;
        if cleared_lines < self.line_cleared_counter.len() {
            self.line_cleared_counter[cleared_lines] += 1;
        }
        self.score += SCORE_TABLE[cleared_lines];
    }
}
