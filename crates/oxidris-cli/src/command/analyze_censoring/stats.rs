//! Survival statistics calculation
//!
//! This module provides structures and functions for calculating survival
//! statistics from censored data, including Kaplan-Meier analysis.

use std::collections::BTreeMap;

use oxidris_stats::survival::KaplanMeierCurve;

use crate::model::session::{BoardAndPlacement, SessionData};

/// Survival statistics for a group of observations
pub(super) struct SurvivalStats {
    /// Total number of observations
    pub boards_count: usize,
    /// Number of censored observations
    pub censored_count: usize,
    /// Mean survival time for complete observations only
    pub mean_complete: f64,
    /// Naive mean survival time for all observations
    pub mean_all: f64,
    /// Kaplan-Meier median survival time (if calculated)
    pub median_km: Option<f64>,
    /// Kaplan-Meier survival curve (if calculated)
    pub km_curve: Option<KaplanMeierCurve>,
}

impl SurvivalStats {
    /// Calculate basic survival statistics from raw data (without KM analysis)
    ///
    /// # Arguments
    /// * `data` - Slice of (`survival_time`, `is_censored`) tuples
    #[expect(clippy::cast_precision_loss)]
    pub(super) fn from_data(data: &[(usize, bool)]) -> Self {
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

        Self {
            boards_count,
            censored_count,
            mean_complete,
            mean_all,
            median_km: None,
            km_curve: None,
        }
    }

    /// Calculate full survival statistics including Kaplan-Meier analysis
    ///
    /// # Arguments
    /// * `data` - Slice of (`survival_time`, `is_censored`) tuples
    pub(super) fn from_data_with_km(data: &[(usize, bool)]) -> Self {
        let mut stats = Self::from_data(data);
        let km = KaplanMeierCurve::from_data(data.to_vec());
        stats.median_km = km.median_survival();
        stats.km_curve = Some(km);
        stats
    }

    /// Censoring rate as percentage
    #[expect(clippy::cast_precision_loss)]
    pub(super) fn censoring_rate(&self) -> f64 {
        100.0 * self.censored_count as f64 / self.boards_count as f64
    }

    /// Ratio of Mean(All) / Mean(Comp) - optimistic bias
    fn all_comp_ratio(&self) -> f64 {
        if self.mean_complete == 0.0 {
            0.0
        } else {
            self.mean_all / self.mean_complete
        }
    }

    /// Ratio formatted as string with warning flag
    pub(super) fn all_comp_ratio_str(&self) -> String {
        let complete_count = self.boards_count - self.censored_count;
        if complete_count == 0 {
            "N/A".to_string()
        } else {
            let ratio = self.all_comp_ratio();
            if ratio > 1.5 {
                format!("⚠{ratio:.2}x")
            } else {
                format!("{ratio:.2}x")
            }
        }
    }

    /// Difference between KM median and naive mean as percentage
    fn km_vs_all_pct(&self) -> Option<f64> {
        self.median_km
            .map(|km| (km - self.mean_all) / self.mean_all * 100.0)
    }

    /// KM vs All formatted as string with sign
    pub(super) fn km_vs_all_str(&self) -> String {
        self.km_vs_all_pct().map_or("N/A".to_string(), |pct| {
            if pct >= 0.0 {
                format!("+{pct:.1}%")
            } else {
                format!("{pct:.1}%")
            }
        })
    }
}

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
/// ```ignore
/// // Group by feature value
/// let data = collect_survival_time_by_group(sessions, |_session, board| {
///     feature.extract_raw(&PlacementAnalysis::from_board(...))
/// });
///
/// // Group by evaluator name
/// let data = collect_survival_time_by_group(sessions, |session, _board| {
///     session.placement_evaluator.clone()
/// });
/// ```
pub(super) fn collect_survival_stats_by_group<F, T>(
    sessions: &[SessionData],
    include_km: bool,
    mut group: F,
) -> BTreeMap<T, SurvivalStats>
where
    F: FnMut(&SessionData, &BoardAndPlacement) -> T,
    T: Ord,
{
    let mut data_map: BTreeMap<T, Vec<(usize, bool)>> = BTreeMap::new();

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

    data_map
        .into_iter()
        .map(|(key, data)| {
            let value = if include_km {
                SurvivalStats::from_data_with_km(&data)
            } else {
                SurvivalStats::from_data(&data)
            };
            (key, value)
        })
        .collect()
}
