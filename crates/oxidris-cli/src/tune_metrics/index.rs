use std::array;

use oxidris_ai::ALL_METRICS;

use super::data::BoardMetrics;

#[derive(Debug, Clone)]
pub struct BoardIndex {
    sorted_indices: [Vec<usize>; ALL_METRICS.len()],
}

impl BoardIndex {
    pub fn new(boards_metrics: &[BoardMetrics]) -> Self {
        Self {
            sorted_indices: array::from_fn(|metric_idx| {
                let mut indices = (0..boards_metrics.len()).collect::<Vec<_>>();
                indices.sort_by(|&a, &b| {
                    let val_a = boards_metrics[a].metrics[metric_idx].normalized;
                    let val_b = boards_metrics[b].metrics[metric_idx].normalized;
                    val_b.total_cmp(&val_a)
                });
                indices
            }),
        }
    }

    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    pub fn get_boards_at_percentile(&self, metric_idx: usize, percentile: f32) -> &[usize] {
        let indices = &self.sorted_indices[metric_idx];
        let total = indices.len() as f32;

        let start_idx = ((percentile / 100.0) * total) as usize;
        let end_idx = (((percentile + 1.0) / 100.0) * total) as usize;
        let end_idx = end_idx.min(indices.len());

        &indices[start_idx..end_idx]
    }

    pub fn get_boards_in_rank_range(
        &self,
        metric_idx: usize,
        start_rank: usize,
        end_rank: usize,
    ) -> &[usize] {
        let indices = &self.sorted_indices[metric_idx];
        let end_rank = end_rank.min(indices.len());
        &indices[start_rank..end_rank]
    }

    pub fn get_board_rank(&self, metric_idx: usize, board_idx: usize) -> Option<usize> {
        self.sorted_indices[metric_idx]
            .iter()
            .position(|&idx| idx == board_idx)
    }
}
