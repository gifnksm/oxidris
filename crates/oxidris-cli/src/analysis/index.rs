use oxidris_evaluator::board_feature::BoxedBoardFeature;

use crate::analysis::BoardSample;

/// Sorted index for efficient board sample lookup by feature values
///
/// Maintains sorted indices for each feature dimension to enable fast
/// percentile-based and rank-based queries. This is particularly useful
/// for selecting representative samples or analyzing extreme cases.
///
/// # Structure
///
/// For each feature, maintains a list of sample indices sorted by
/// normalized feature value in descending order (highest value first).
///
/// # Query Operations
///
/// - **Percentile lookup**: Find samples at specific percentiles (e.g., top 5%)
/// - **Rank-based filtering**: Get samples in a rank range (e.g., ranks 0-99)
/// - **Board ranking**: Find the rank of a specific sample
///
/// # Performance
///
/// - Construction: O(n·f·log(n)) where n=samples, f=features
/// - Queries: O(1) for rank ranges, O(log n) for rank lookup
///
/// # Examples
///
/// ```no_run
/// use oxidris_cli::analysis::BoardIndex;
/// # let features = todo!();
/// # let samples = todo!();
/// # let feature_idx = 0;
/// # let board_idx = 0;
/// let index = BoardIndex::from_samples(&features, &samples);
///
/// // Get top 10 boards for a feature
/// let top_10 = index.get_boards_in_rank_range(feature_idx, 0, 10);
///
/// // Get boards in 95th percentile
/// let top_5_percent = index.get_boards_at_percentile(feature_idx, 95.0);
///
/// // Find rank of a specific board
/// if let Some(rank) = index.get_board_rank(feature_idx, board_idx) {
///     println!("Board is ranked #{}", rank);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BoardIndex {
    /// Sorted indices for each feature (outer: feature index, inner: sorted sample indices)
    sorted_indices: Vec<Vec<usize>>,
}

impl BoardIndex {
    /// Build an index from board samples
    ///
    /// Sorts board indices by normalized feature values (descending order)
    /// to enable efficient percentile-based and rank-based queries.
    ///
    /// # Arguments
    ///
    /// * `features` - Feature definitions (determines number of indices)
    /// * `board_samples` - Samples to index
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidris_cli::analysis::BoardIndex;
    /// # let features = todo!();
    /// # let samples = todo!();
    /// let index = BoardIndex::from_samples(&features, &samples);
    /// // Index maintains one sorted list per feature
    /// ```
    pub fn from_samples(features: &[BoxedBoardFeature], board_samples: &[BoardSample]) -> Self {
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

    /// Get board indices at a specific percentile for a feature
    ///
    /// Returns a slice of board indices corresponding to the given percentile.
    /// For example, `percentile = 95.0` returns boards in the 95th percentile
    /// (approximately the top 5% by normalized feature value).
    ///
    /// # Arguments
    ///
    /// * `feature_idx` - Index of the feature dimension
    /// * `percentile` - Percentile value (0.0 to 100.0)
    ///
    /// # Returns
    ///
    /// A slice of board sample indices at the specified percentile.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidris_cli::analysis::BoardIndex;
    /// # let index = todo!();
    /// # let holes_feature_idx = 0;
    /// // Get boards with worst hole counts (95th percentile = most holes)
    /// let worst = index.get_boards_at_percentile(holes_feature_idx, 95.0);
    /// println!("Found {} boards with severe hole problems", worst.len());
    /// ```
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

    /// Get board indices in a rank range for a feature
    ///
    /// Returns boards ranked from `start_rank` (inclusive) to `end_rank` (exclusive).
    /// Rank 0 corresponds to the board with the highest normalized feature value.
    ///
    /// # Arguments
    ///
    /// * `feature_idx` - Index of the feature dimension
    /// * `start_rank` - Starting rank (inclusive, 0-based)
    /// * `end_rank` - Ending rank (exclusive, 0-based)
    ///
    /// # Returns
    ///
    /// A slice of board sample indices in the specified rank range.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidris_cli::analysis::BoardIndex;
    /// # let index = todo!();
    /// # let height_feature_idx = 0;
    /// // Get top 10 boards by height
    /// let top_10 = index.get_boards_in_rank_range(height_feature_idx, 0, 10);
    ///
    /// // Get boards ranked 100-199
    /// let mid_tier = index.get_boards_in_rank_range(height_feature_idx, 100, 200);
    /// ```
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

    /// Get the rank of a specific board for a feature
    ///
    /// Returns the rank (0-based) of the board, where rank 0 is the highest
    /// normalized feature value. Returns `None` if the board index is not found.
    ///
    /// # Arguments
    ///
    /// * `feature_idx` - Index of the feature dimension
    /// * `board_idx` - Sample index to find
    ///
    /// # Returns
    ///
    /// The rank of the board, or `None` if not found.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidris_cli::analysis::BoardIndex;
    /// # let index = todo!();
    /// # let feature_idx = 0;
    /// # let sample_idx = 42;
    /// # let samples_len = 1000;
    /// if let Some(rank) = index.get_board_rank(feature_idx, sample_idx) {
    ///     println!("This board is ranked #{} out of {}", rank, samples_len);
    /// }
    /// ```
    #[expect(unused, reason = "may be used later")] // TODO
    pub fn get_board_rank(&self, feature_idx: usize, board_idx: usize) -> Option<usize> {
        self.sorted_indices[feature_idx]
            .iter()
            .position(|&idx| idx == board_idx)
    }
}
