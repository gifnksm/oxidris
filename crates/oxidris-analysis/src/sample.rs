//! Feature sample extraction from gameplay sessions
//!
//! This module provides types for extracting and storing feature values from
//! board states in gameplay sessions.
//!
//! # Overview
//!
//! Feature sampling is the process of computing feature values from board states
//! to create datasets for analysis and training. This module provides two sample types:
//!
//! - [`RawBoardSample`]: Raw feature values only (for statistics computation)
//! - [`BoardSample`]: Full feature vectors (raw, transformed, normalized)
//!
//! # Sample Types
//!
//! ## `RawBoardSample`
//!
//! Contains only raw feature values extracted by `BoardFeatureSource`. Used for:
//!
//! - Computing statistics before feature construction
//! - Calculating normalization parameters
//! - Lightweight data collection
//!
//! ## `BoardSample`
//!
//! Contains complete feature vectors with raw, transformed, and normalized values.
//! Used for:
//!
//! - Distribution analysis at each processing stage
//! - Training data visualization
//! - Feature engineering validation
//! - Outlier detection
//!
//! # Typical Workflow
//!
//! ```text
//! Session Data
//!     ↓
//! Extract Raw Samples (RawBoardSample)
//!     ↓
//! Compute Statistics
//!     ↓
//! Build Features
//!     ↓
//! Extract Full Samples (BoardSample)
//!     ↓
//! Analyze/Visualize
//! ```
//!
//! # Examples
//!
//! ## Extract Raw Samples for Statistics
//!
//! ```no_run
//! use oxidris_analysis::{sample::RawBoardSample, session::SessionData};
//! use oxidris_evaluator::board_feature;
//!
//! let sessions: Vec<SessionData> = vec![]; // Load from file
//! let sources = board_feature::all_board_feature_sources();
//!
//! // Extract raw values for all boards
//! let raw_samples = RawBoardSample::from_sessions(&sources, &sessions);
//!
//! println!("Extracted {} raw samples", raw_samples.len());
//! println!(
//!     "First sample has {} features",
//!     raw_samples[0].raw_values.len()
//! );
//! ```
//!
//! ## Extract Full Samples for Analysis
//!
//! ```no_run
//! use oxidris_analysis::{sample::BoardSample, session::SessionData};
//! use oxidris_evaluator::board_feature::BoxedBoardFeature;
//!
//! let sessions: Vec<SessionData> = vec![]; // Load from file
//! let features: Vec<BoxedBoardFeature> = vec![]; // Build features
//!
//! // Extract full feature vectors
//! let samples = BoardSample::from_sessions(&features, &sessions);
//!
//! // Analyze distributions at each stage
//! for sample in &samples {
//!     for (i, fv) in sample.feature_vector.iter().enumerate() {
//!         println!(
//!             "Feature {}: raw={}, transformed={:.2}, normalized={:.2}",
//!             i, fv.raw, fv.transformed, fv.normalized
//!         );
//!     }
//! }
//! ```

use oxidris_evaluator::{
    board_feature::{BoardFeatureValue, BoxedBoardFeature, BoxedBoardFeatureSource},
    placement_analysis::PlacementAnalysis,
};

use crate::session::{BoardAndPlacement, SessionData};

/// Raw feature value sample (no normalization)
///
/// Contains only raw feature values extracted from board sources.
/// Used for computing statistics before normalization.
#[derive(Debug, Clone)]
pub struct RawBoardSample {
    /// The original board state and placement
    pub board: BoardAndPlacement,
    /// Raw feature values (one per source)
    pub raw_values: Vec<u32>,
}

impl RawBoardSample {
    /// Create a raw feature sample from board sources
    ///
    /// Extracts raw values without any transformation or normalization.
    /// This is used for computing statistics.
    #[must_use]
    pub fn from_board(sources: &[BoxedBoardFeatureSource], board: &BoardAndPlacement) -> Self {
        let analysis = PlacementAnalysis::from_board(&board.before_placement, board.placement);
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
    #[must_use]
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
/// use oxidris_analysis::{
///     sample::BoardSample,
///     session::{BoardAndPlacement, SessionData},
/// };
/// use oxidris_evaluator::board_feature::BoxedBoardFeature;
///
/// let features: Vec<BoxedBoardFeature> = todo!();
/// let sessions: Vec<SessionData> = todo!();
/// let board: BoardAndPlacement = todo!();
///
/// // Create a sample from a single board
/// let sample = BoardSample::from_board(&features, &board);
///
/// // Collect samples from session data
/// let samples = BoardSample::from_sessions(&features, &sessions);
/// ```
#[derive(Debug, Clone)]
pub struct BoardSample {
    /// The original board state and placement
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
    /// use oxidris_analysis::{sample::BoardSample, session::BoardAndPlacement};
    /// use oxidris_evaluator::board_feature::BoxedBoardFeature;
    ///
    /// let all_features: Vec<BoxedBoardFeature> = todo!();
    /// let board_and_placement: BoardAndPlacement = todo!();
    ///
    /// let sample = BoardSample::from_board(&all_features, &board_and_placement);
    /// assert_eq!(sample.feature_vector.len(), all_features.len());
    /// ```
    #[must_use]
    pub fn from_board(features: &[BoxedBoardFeature], board: &BoardAndPlacement) -> Self {
        let analysis = PlacementAnalysis::from_board(&board.before_placement, board.placement);
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
    /// use oxidris_analysis::{sample::BoardSample, session::SessionCollection};
    /// use oxidris_evaluator::board_feature::BoxedBoardFeature;
    ///
    /// let features: Vec<BoxedBoardFeature> = todo!();
    /// let collection: SessionCollection = todo!();
    ///
    /// let samples = BoardSample::from_sessions(&features, &collection.sessions);
    /// println!("Collected {} samples", samples.len());
    /// ```
    #[must_use]
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
