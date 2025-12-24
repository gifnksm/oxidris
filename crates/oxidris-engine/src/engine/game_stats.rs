const SCORE_TABLE: [usize; 5] = [0, 100, 300, 500, 800];

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
    #[must_use]
    pub const fn new() -> Self {
        Self {
            score: 0,
            completed_pieces: 0,
            total_cleared_lines: 0,
            line_cleared_counter: [0; 5],
        }
    }

    #[must_use]
    pub const fn score(&self) -> usize {
        self.score
    }

    #[must_use]
    pub fn level(&self) -> usize {
        self.total_cleared_lines / 10
    }

    #[must_use]
    pub const fn completed_pieces(&self) -> usize {
        self.completed_pieces
    }

    #[must_use]
    pub const fn total_cleared_lines(&self) -> usize {
        self.total_cleared_lines
    }

    #[must_use]
    pub const fn line_cleared_counter(&self) -> &[usize; 5] {
        &self.line_cleared_counter
    }

    pub const fn complete_piece_drop(&mut self, cleared_lines: usize) {
        self.completed_pieces += 1;
        self.total_cleared_lines += cleared_lines;
        if cleared_lines < self.line_cleared_counter.len() {
            self.line_cleared_counter[cleared_lines] += 1;
        }
        self.score += SCORE_TABLE[cleared_lines];
    }
}
