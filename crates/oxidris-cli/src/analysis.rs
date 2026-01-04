use oxidris_evaluator::{
    board_feature::{BoardFeatureValue, BoxedBoardFeatureSource},
    placement_analysis::PlacementAnalysis,
};
use oxidris_stats::comprehensive::ComprehensiveStats;

use crate::data::{BoardAndPlacement, BoardFeatureStatistics, BoardSample, SessionData};

pub fn extract_all_board_features(
    features: &[BoxedBoardFeatureSource],
    sessions: &[SessionData],
) -> Vec<BoardSample> {
    sessions
        .iter()
        .flat_map(|session| &session.boards)
        .map(|board| extract_board_features(features, board))
        .collect()
}

fn extract_board_features(
    features: &[BoxedBoardFeatureSource],
    board: &BoardAndPlacement,
) -> BoardSample {
    let analysis = PlacementAnalysis::from_board(&board.board, board.placement);
    let feature_vector = features
        .iter()
        .map(|feature| feature.compute_feature_value(&analysis))
        .collect();
    BoardSample {
        board: board.clone(),
        feature_vector,
    }
}

pub fn coimpute_statistics(
    features: &[BoxedBoardFeatureSource],
    board_samples: &[BoardSample],
) -> Vec<BoardFeatureStatistics> {
    (0..features.len())
        .map(|i| compute_feature_statistics(&board_samples.iter().map(|bf| bf.feature_vector[i])))
        .collect()
}

#[expect(clippy::cast_precision_loss)]
fn compute_feature_statistics<I>(values: &I) -> BoardFeatureStatistics
where
    I: ExactSizeIterator<Item = BoardFeatureValue> + Clone,
{
    let raw_values = values.clone().map(|f| f.raw as f32);
    let transformed_values = values.clone().map(|f| f.transformed);
    let normalized_values = values.clone().map(|f| f.normalized);

    BoardFeatureStatistics {
        raw: compute_value_stats(raw_values, 11, Some(0.0), None, Some(1.0)),
        transformed: compute_value_stats(transformed_values, 11, Some(0.0), None, None),
        normalized: compute_value_stats(normalized_values, 11, Some(0.0), Some(1.0), Some(0.1)),
    }
}

fn compute_value_stats<I>(
    values: I,
    hist_num_bins: usize,
    hist_min: Option<f32>,
    hist_max: Option<f32>,
    hist_bin_width_unit: Option<f32>,
) -> ComprehensiveStats
where
    I: ExactSizeIterator<Item = f32>,
{
    let mut values = values.collect::<Vec<_>>();
    values.sort_by(f32::total_cmp);

    ComprehensiveStats::from_sorted(
        &values,
        &[1.0, 5.0, 10.0, 25.0, 50.0, 75.0, 90.0, 95.0, 99.0],
        hist_num_bins,
        hist_min,
        hist_max,
        hist_bin_width_unit,
    )
    .unwrap()
}
