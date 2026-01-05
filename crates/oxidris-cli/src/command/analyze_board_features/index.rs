use oxidris_evaluator::board_feature::BoxedBoardFeature;

use crate::model::session::BoardSample;

#[derive(Debug, Clone)]
pub struct BoardIndex {
    sorted_indices: Vec<Vec<usize>>,
}

impl BoardIndex {
    pub fn new(features: &[BoxedBoardFeature], board_samples: &[BoardSample]) -> Self {
        Self {
            sorted_indices: (0..features.len())
                .map(|feature_idx| {
                    let mut indices = (0..board_samples.len()).collect::<Vec<_>>();
                    indices.sort_by(|&a, &b| {
                        let val_a = board_samples[a].feature_vector[feature_idx].normalized;
                        let val_b = board_samples[b].feature_vector[feature_idx].normalized;
                        val_b.total_cmp(&val_a)
                    });
                    indices
                })
                .collect(),
        }
    }

    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    #[expect(unused, reason = "may be used later")] // TODO
    pub fn get_boards_at_percentile(&self, feature_idx: usize, percentile: f32) -> &[usize] {
        let indices = &self.sorted_indices[feature_idx];
        let total = indices.len() as f32;

        let start_idx = ((percentile / 100.0) * total) as usize;
        let end_idx = (((percentile + 1.0) / 100.0) * total) as usize;
        let end_idx = end_idx.min(indices.len());

        &indices[start_idx..end_idx]
    }

    #[expect(unused, reason = "may be used later")] // TODO
    pub fn get_boards_in_rank_range(
        &self,
        feature_idx: usize,
        start_rank: usize,
        end_rank: usize,
    ) -> &[usize] {
        let indices = &self.sorted_indices[feature_idx];
        let end_rank = end_rank.min(indices.len());
        &indices[start_rank..end_rank]
    }

    #[expect(unused, reason = "may be used later")] // TODO
    pub fn get_board_rank(&self, feature_idx: usize, board_idx: usize) -> Option<usize> {
        self.sorted_indices[feature_idx]
            .iter()
            .position(|&idx| idx == board_idx)
    }
}
