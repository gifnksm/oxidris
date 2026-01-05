use oxidris_evaluator::{
    board_feature::{BoardFeatureValue, BoxedBoardFeature, BoxedBoardFeatureSource},
    placement_analysis::PlacementAnalysis,
};

use crate::model::session::{BoardAndPlacement, SessionData};

/// Raw feature value sample (no normalization)
///
/// Contains only raw feature values extracted from board sources.
/// Used for computing statistics before normalization.
#[derive(Debug, Clone)]
pub struct RawBoardSample {
    /// The original board state and placement
    #[expect(unused, reason = "may be used later")]
    pub board: BoardAndPlacement,
    /// Raw feature values (one per source)
    pub raw_values: Vec<u32>,
}

impl RawBoardSample {
    /// Create a raw feature sample from board sources
    ///
    /// Extracts raw values without any transformation or normalization.
    /// This is used for computing statistics.
    pub fn from_board(sources: &[BoxedBoardFeatureSource], board: &BoardAndPlacement) -> Self {
        let analysis = PlacementAnalysis::from_board(&board.board, board.placement);
        let raw_values = sources
            .iter()
            .map(|source| source.extract_raw(&analysis))
            .collect();
        Self {
            board: board.clone(),
            raw_values,
        }
    }

    /// Collect raw feature samples from session data
    pub fn from_sessions(
        sources: &[BoxedBoardFeatureSource],
        sessions: &[SessionData],
    ) -> Vec<RawBoardSample> {
        sessions
            .iter()
            .flat_map(|session| &session.boards)
            .map(|board| Self::from_board(sources, board))
            .collect()
    }
}

/// A board observation with computed feature values
///
/// Represents a single sample in feature space, containing both the original
/// board state and its computed feature vector. This is an intermediate
/// analysis data structure and is not serialized.
///
/// # Feature Vector
///
/// Each element in the feature vector contains three values:
///
/// - **Raw**: The direct measurement from the board (e.g., number of holes)
/// - **Transformed**: The raw value after non-linear transformation
/// - **Normalized**: The transformed value scaled to [0, 1] range
///
/// # Examples
///
/// ```no_run
/// use oxidris_evaluator::analysis::BoardSample;
/// # let features = todo!();
/// # let board = todo!();
/// // Create a sample from a single board
/// let sample = BoardSample::from_board(&features, &board);
///
/// // Collect samples from session data
/// let samples = BoardSample::from_sessions(&features, &sessions);
/// ```
#[derive(Debug, Clone)]
pub struct BoardSample {
    /// The original board state and placement
    #[expect(unused, reason = "may be used later")] // TODO
    pub board: BoardAndPlacement,
    /// Computed feature values (raw, transformed, normalized)
    pub feature_vector: Vec<BoardFeatureValue>,
}

impl BoardSample {
    /// Create a feature sample from a board
    ///
    /// Analyzes the board placement and computes feature values
    /// (raw, transformed, and normalized) to create a sample observation.
    ///
    /// # Arguments
    ///
    /// * `features` - Feature definitions to compute
    /// * `board` - Board state and placement to analyze
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidris_evaluator::analysis::BoardSample;
    /// # let all_features = todo!();
    /// # let board_and_placement = todo!();
    /// let sample = BoardSample::from_board(&all_features, &board_and_placement);
    /// assert_eq!(sample.feature_vector.len(), all_features.len());
    /// ```
    pub fn from_board(features: &[BoxedBoardFeature], board: &BoardAndPlacement) -> Self {
        let analysis = PlacementAnalysis::from_board(&board.board, board.placement);
        let feature_vector = features
            .iter()
            .map(|feature| feature.compute_feature_value(&analysis))
            .collect();
        Self {
            board: board.clone(),
            feature_vector,
        }
    }

    /// Collect feature samples from session data
    ///
    /// Processes all boards from all sessions to create a sample collection.
    /// This flattens boards across all sessions into a single vector of samples.
    ///
    /// # Arguments
    ///
    /// * `features` - Feature definitions to compute
    /// * `sessions` - Game sessions containing board states
    ///
    /// # Returns
    ///
    /// A vector of samples, one for each board in the sessions.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidris_evaluator::analysis::BoardSample;
    /// # let features = todo!();
    /// # let collection = todo!();
    /// let samples = BoardSample::from_sessions(&features, &collection.sessions);
    /// println!("Collected {} samples", samples.len());
    /// ```
    pub fn from_sessions(
        features: &[BoxedBoardFeature],
        sessions: &[SessionData],
    ) -> Vec<BoardSample> {
        sessions
            .iter()
            .flat_map(|session| &session.boards)
            .map(|board| Self::from_board(features, board))
            .collect()
    }
}
