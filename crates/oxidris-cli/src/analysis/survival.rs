//! Survival statistics calculation
//!
//! This module provides structures and functions for calculating survival
//! statistics from censored data, including Kaplan-Meier analysis.

use std::collections::BTreeMap;

use oxidris_stats::survival::KaplanMeierCurve;

use crate::model::session::{BoardAndPlacement, SessionData};

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
    /// # Arguments
    /// * `sessions` - Slice of session data containing board states
    /// * `group` - Closure that computes the grouping key from session and board
    ///
    /// # Returns
    /// A map from group key to list of `(survival_time, is_censored)` tuples
    ///
    /// # Examples
    /// ```no_run
    /// # let sessions = vec![];
    /// // Group by feature value
    /// let data = SurvivalStats::collect_by_group(sessions, |_session, board| {
    ///     feature.extract_raw(&PlacementAnalysis::from_board(...))
    /// });
    ///
    /// // Group by evaluator name
    /// let data = collect_survival_time_by_group(sessions, |session, _board| {
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
    pub fn filter_by_percentiles(&self, percentiles: &[f64]) -> BTreeMap<&K, (f64, &SurvivalStats)>
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
                percentile_values.insert(value, (percentiles[percentile_idx], stats));
                percentile_idx += 1;
            }
        }

        percentile_values
    }
}
