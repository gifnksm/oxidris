//! Survival analysis for right-censored training data
//!
//! This module provides structures and functions for calculating survival
//! statistics from censored data using Kaplan-Meier estimation.
//!
//! # Overview
//!
//! In Tetris AI training, games may end in two ways:
//!
//! - **Complete**: Game over due to terminal board state (death observed)
//! - **Censored**: Game reached maximum turn limit (death not observed)
//!
//! Standard statistics (mean, median) are biased when applied to censored data,
//! underestimating survival time for good board states. Kaplan-Meier estimation
//! provides unbiased survival estimates by properly accounting for censoring.
//!
//! # Right-Censored Data
//!
//! Games that survive to `MAX_TURNS` are "right-censored" - we know they survived
//! at least that long, but not their true survival time:
//!
//! ```text
//! Complete:  |----x  (died at 50 turns)
//! Censored:  |-------> (survived past 500 turns, true survival unknown)
//! ```
//!
//! **Problem**: Better board states are more likely to be censored, creating bias.
//!
//! **Solution**: Kaplan-Meier estimator handles censoring to produce unbiased
//! survival curves and median survival times.
//!
//! # Use Cases
//!
//! - **Feature Engineering**: Transform feature values through survival time
//! - **Evaluator Comparison**: Compare AI performance accounting for censoring
//! - **Data Quality**: Understand censoring patterns in training data
//! - **Normalization**: Compute KM-based normalization parameters
//!
//! # Examples
//!
//! ## Basic Survival Statistics
//!
//! ```no_run
//! use oxidris_analysis::survival::SurvivalStats;
//!
//! // Survival data: (survival_time, is_censored)
//! let data = vec![
//!     (45, false),  // died at 45 turns
//!     (500, true),  // censored at 500 turns
//!     (123, false), // died at 123 turns
//!     (500, true),  // censored at 500 turns
//! ];
//!
//! let stats = SurvivalStats::from_data(&data);
//!
//! println!("Total observations: {}", stats.boards_count);
//! println!("Censored: {}", stats.censored_count);
//! println!("Naive mean: {:.1}", stats.mean_all); // Biased
//! if let Some(km_median) = stats.median_km {
//!     println!("KM median survival: {:.1}", km_median); // Unbiased
//! }
//! ```
//!
//! ## Group by Feature Value
//!
//! ```no_run
//! use oxidris_analysis::{session::SessionData, survival::SurvivalStatsMap};
//! use oxidris_evaluator::{
//!     board_feature::{BoardFeatureSource, source::NumHoles},
//!     placement_analysis::PlacementAnalysis,
//! };
//!
//! let sessions: Vec<SessionData> = vec![]; // Load from file
//! let source = NumHoles;
//!
//! // Group survival times by number of holes
//! let stats_map = SurvivalStatsMap::collect_by_group(&sessions, |_session, board| {
//!     let analysis = PlacementAnalysis::from_board(&board.before_placement, board.placement);
//!     source.extract_raw(&analysis)
//! });
//!
//! // Analyze survival by hole count
//! for (holes, stats) in &stats_map.map {
//!     if let Some(km_median) = stats.median_km {
//!         println!("Holes={}: KM median = {:.1} turns", holes, km_median);
//!     }
//! }
//! ```
//!
//! ## Compute Percentile Values
//!
//! ```no_run
//! use oxidris_analysis::{session::SessionData, survival::SurvivalStatsMap};
//! # use oxidris_evaluator::{
//! #     board_feature::{source::NumHoles, BoardFeatureSource},
//! #     placement_analysis::PlacementAnalysis,
//! # };
//! # let sessions: Vec<SessionData> = vec![];
//! # let source = NumHoles;
//! # let stats_map = SurvivalStatsMap::collect_by_group(&sessions, |_session, board| {
//! #     let analysis = PlacementAnalysis::from_board(&board.before_placement, board.placement);
//! #     source.extract_raw(&analysis)
//! # });
//!
//! // Get feature values at P05 and P95 percentiles
//! let percentiles = stats_map.filter_by_percentiles(&[0.05, 0.95]);
//!
//! for (value, (percentiles, stats)) in &percentiles {
//!     let percentile_labels = percentiles
//!         .iter()
//!         .map(|p| format!("P{:.0}", p * 100.0))
//!         .collect::<Vec<_>>()
//!         .join("/");
//!     println!(
//!         "{}: value={}, KM median={:.1}",
//!         percentile_labels,
//!         value,
//!         stats.median_km.unwrap_or(0.0)
//!     );
//! }
//! ```

use std::collections::BTreeMap;

use oxidris_evaluator::{board_feature::BoardFeatureSource, placement_analysis::PlacementAnalysis};
use oxidris_stats::survival::KaplanMeierCurve;

use crate::session::{BoardAndPlacement, SessionData};

/// Survival statistics for a group of observations
#[derive(Debug, Clone)]
pub struct SurvivalStats {
    /// Total number of observations
    pub boards_count: usize,
    /// Number of censored observations
    pub censored_count: usize,
    /// Mean survival time for complete observations only
    pub mean_complete: f64,
    /// Naive mean survival time for all observations
    pub mean_all: f64,
    /// Kaplan-Meier median survival time
    pub median_km: Option<f64>,
    /// Kaplan-Meier survival curve
    pub km_curve: KaplanMeierCurve,
}

#[derive(Debug, Clone)]
pub struct SurvivalStatsMap<K> {
    pub map: BTreeMap<K, SurvivalStats>,
}

impl SurvivalStats {
    /// Calculate basic survival statistics from raw data (without KM analysis)
    ///
    /// # Arguments
    /// * `data` - Slice of (`survival_time`, `is_censored`) tuples
    #[expect(clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_data(data: &[(usize, bool)]) -> Self {
        let boards_count = data.len();
        let censored_count = data.iter().filter(|(_, c)| *c).count();

        let complete_remaining: Vec<usize> =
            data.iter().filter(|(_, c)| !*c).map(|(r, _)| *r).collect();

        let all_remaining: Vec<usize> = data.iter().map(|(r, _)| *r).collect();

        let mean_complete = if complete_remaining.is_empty() {
            0.0
        } else {
            complete_remaining.iter().sum::<usize>() as f64 / complete_remaining.len() as f64
        };

        let mean_all = all_remaining.iter().sum::<usize>() as f64 / all_remaining.len() as f64;

        let km_curve = KaplanMeierCurve::from_data(data.to_vec());
        let median_km = km_curve.median_survival();

        Self {
            boards_count,
            censored_count,
            mean_complete,
            mean_all,
            median_km,
            km_curve,
        }
    }
}

impl<K> SurvivalStatsMap<K> {
    /// Collect survival time data grouped by a custom key
    ///
    /// This is a generic data collection function that extracts survival time
    /// observations from session data, grouped by any arbitrary key computed
    /// from the session and board state.
    ///
    /// For each board in each session, computes:
    /// - **Group key**: Computed by the provided closure (e.g., feature value)
    /// - **Survival time**: Remaining turns until session end (`game_end - board.turn`)
    /// - **Censoring status**: Whether session ended in game over or max turns
    ///
    /// # Arguments
    ///
    /// * `sessions` - Slice of session data containing board states
    /// * `group` - Closure that computes the grouping key from session and board
    ///
    /// # Returns
    ///
    /// A map from group key to [`SurvivalStats`] with KM analysis results
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxidris_analysis::{session::SessionData, survival::SurvivalStatsMap};
    /// use oxidris_evaluator::{
    ///     board_feature::{BoardFeatureSource, source::NumHoles},
    ///     placement_analysis::PlacementAnalysis,
    /// };
    ///
    /// let sessions: Vec<SessionData> = vec![];
    /// let source = NumHoles;
    ///
    /// // Group by number of holes
    /// let stats = SurvivalStatsMap::collect_by_group(&sessions, |_session, board| {
    ///     let analysis = PlacementAnalysis::from_board(&board.before_placement, board.placement);
    ///     source.extract_raw(&analysis)
    /// });
    ///
    /// // Access KM median for each hole count
    /// for (holes, stat) in &stats.map {
    ///     if let Some(km_median) = stat.median_km {
    ///         println!("Holes={}: KM median survival = {:.1}", holes, km_median);
    ///     }
    /// }
    /// ```
    ///
    /// ## Group by Evaluator Name
    ///
    /// ```no_run
    /// use oxidris_analysis::{session::SessionData, survival::SurvivalStatsMap};
    ///
    /// let sessions: Vec<SessionData> = vec![];
    ///
    /// // Group by evaluator name
    /// let data = SurvivalStatsMap::collect_by_group(&sessions, |session, _board| {
    ///     session.placement_evaluator.clone()
    /// });
    /// ```
    pub fn collect_by_group<F>(sessions: &[SessionData], mut group: F) -> Self
    where
        F: FnMut(&SessionData, &BoardAndPlacement) -> K,
        K: Ord,
    {
        let mut data_map: BTreeMap<K, Vec<(usize, bool)>> = BTreeMap::new();

        for session in sessions {
            let is_censored = !session.is_game_over;
            let game_end = session.survived_turns;

            for board in &session.boards {
                let key = group(session, board);
                let survival_time = game_end - board.turn;
                data_map
                    .entry(key)
                    .or_default()
                    .push((survival_time, is_censored));
            }
        }

        Self {
            map: data_map
                .into_iter()
                .map(|(key, data)| (key, SurvivalStats::from_data(&data)))
                .collect(),
        }
    }

    #[expect(clippy::cast_precision_loss)]
    #[must_use]
    pub fn filter_by_percentiles(
        &self,
        percentiles: &[f64],
    ) -> BTreeMap<&K, (Vec<f64>, &SurvivalStats)>
    where
        K: Ord,
    {
        let total_boards = self
            .map
            .values()
            .map(|stats| stats.boards_count)
            .sum::<usize>();

        let mut cumulative_boards = 0;
        let mut percentile_values = BTreeMap::new();
        let mut percentile_idx = 0;

        for (value, stats) in &self.map {
            cumulative_boards += stats.boards_count;
            let current_percentile = cumulative_boards as f64 / total_boards as f64;

            while percentile_idx < percentiles.len()
                && current_percentile >= percentiles[percentile_idx]
            {
                percentile_values
                    .entry(value)
                    .or_insert_with(|| (vec![], stats))
                    .0
                    .push(percentiles[percentile_idx]);
                percentile_idx += 1;
            }
        }

        percentile_values
    }
}

impl SurvivalStatsMap<u32> {
    /// Collect survival statistics grouped by a specific board feature value
    ///
    /// This function extracts survival time data from session data,
    /// grouping observations by the raw value of a provided board feature.
    pub fn collect_by_feature_value(
        sessions: &[SessionData],
        feature: &dyn BoardFeatureSource,
    ) -> Self {
        Self::collect_by_group(sessions, |_, board| {
            let analysis = PlacementAnalysis::from_board(&board.before_placement, board.placement);
            feature.extract_raw(&analysis)
        })
    }

    /// Collect survival statistics for multiple board features
    ///
    /// This function processes a list of board feature sources,
    /// collecting survival statistics grouped by each feature's raw value.
    pub fn collect_all_by_feature_value<F>(sessions: &[SessionData], features: &[F]) -> Vec<Self>
    where
        F: AsRef<dyn BoardFeatureSource>,
    {
        // FIXME: This computes PlacementAnalysis multiple times per board
        features
            .iter()
            .map(|feature| Self::collect_by_feature_value(sessions, feature.as_ref()))
            .collect()
    }
}
