/// Kaplan-Meier survival curve for survival analysis.
///
/// The Kaplan-Meier estimator is a non-parametric statistic used to estimate the survival
/// function from lifetime data. It accounts for censored data (observations where the event
/// of interest has not occurred by the end of the study period).
///
/// # Fields
///
/// The curve stores parallel vectors representing the survival function at discrete time points:
/// - Time points where events occurred
/// - Survival probability at each time point
/// - Number of subjects at risk at each time point
/// - Number of events (non-censored observations) at each time point
#[derive(Debug, Clone)]
pub struct KaplanMeierCurve {
    /// Time points where events (non-censored observations) occurred.
    pub times: Vec<usize>,
    /// Survival probability at each corresponding time point.
    /// Values range from 0.0 (no survival) to 1.0 (complete survival).
    pub survival_prob: Vec<f64>,
    /// Number of subjects at risk (not yet experienced the event or censored) at each time point.
    pub at_risk: Vec<usize>,
    /// Number of events (non-censored observations) that occurred at each time point.
    pub events: Vec<usize>,
}

impl KaplanMeierCurve {
    /// Computes the Kaplan-Meier survival curve from survival data.
    ///
    /// # Arguments
    ///
    /// * `data` - A vector of tuples where each tuple contains:
    ///   - `time`: The time at which the observation occurred
    ///   - `is_censored`: `true` if the observation was censored (event did not occur),
    ///     `false` if the event occurred
    ///
    /// # Returns
    ///
    /// A `KaplanMeierCurve` with survival probabilities calculated at each event time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxidris_stats::survival::KaplanMeierCurve;
    /// // Data: (time, is_censored)
    /// let data = vec![
    ///     (10, false), // Event at time 10
    ///     (20, true),  // Censored at time 20
    ///     (30, false), // Event at time 30
    /// ];
    /// let curve = KaplanMeierCurve::from_data(data);
    /// assert!(!curve.times.is_empty());
    /// ```
    #[expect(clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_data(mut data: Vec<(usize, bool)>) -> Self {
        if data.is_empty() {
            return Self {
                times: vec![],
                survival_prob: vec![],
                at_risk: vec![],
                events: vec![],
            };
        }

        // Sort by time
        data.sort_by_key(|(time, _)| *time);

        let mut times = vec![];
        let mut survival_prob = vec![];
        let mut at_risk_vec = vec![];
        let mut events_vec = vec![];

        let mut current_survival = 1.0;
        let total = data.len();

        let mut i = 0;
        while i < data.len() {
            let current_time = data[i].0;
            let at_risk = total - i;

            // Count events (non-censored) at this time point
            let mut event_count = 0;
            let mut j = i;
            while j < data.len() && data[j].0 == current_time {
                if !data[j].1 {
                    // Not censored = event occurred
                    event_count += 1;
                }
                j += 1;
            }

            if event_count > 0 {
                // Update survival probability
                let survival_rate = 1.0 - (event_count as f64 / at_risk as f64);
                current_survival *= survival_rate;

                times.push(current_time);
                survival_prob.push(current_survival);
                at_risk_vec.push(at_risk);
                events_vec.push(event_count);
            }

            i = j;
        }

        Self {
            times,
            survival_prob,
            at_risk: at_risk_vec,
            events: events_vec,
        }
    }

    /// Returns the median survival time.
    ///
    /// The median survival time is the time at which the survival probability
    /// drops to or below 50%. If the survival probability never reaches 50%,
    /// this method returns `None`.
    ///
    /// Linear interpolation is used between time points for more accurate estimates.
    ///
    /// # Returns
    ///
    /// * `Some(time)` - The median survival time if the survival probability reaches 50%
    /// * `None` - If the survival probability never drops to 50% or if the curve is empty
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxidris_stats::survival::KaplanMeierCurve;
    /// let data = vec![(10, false), (20, false), (30, false)];
    /// let curve = KaplanMeierCurve::from_data(data);
    /// if let Some(median) = curve.median_survival() {
    ///     println!("Median survival time: {}", median);
    /// }
    /// ```
    #[expect(clippy::cast_precision_loss)]
    #[must_use]
    pub fn median_survival(&self) -> Option<f64> {
        if self.survival_prob.is_empty() {
            return None;
        }

        // Find first time where survival prob <= 0.5
        for i in 0..self.survival_prob.len() {
            if self.survival_prob[i] <= 0.5 {
                if i == 0 {
                    return Some(self.times[0] as f64);
                }
                // Linear interpolation between points
                let t0 = self.times[i - 1] as f64;
                let t1 = self.times[i] as f64;
                let s0 = self.survival_prob[i - 1];
                let s1 = self.survival_prob[i];
                let median = t0 + (0.5 - s0) / (s1 - s0) * (t1 - t0);
                return Some(median);
            }
        }

        // Survival probability never drops to 50%
        None
    }

    /// Returns the survival probability at a specific time.
    ///
    /// This method uses a step function: the survival probability remains constant
    /// between event times and decreases only when an event occurs.
    ///
    /// # Arguments
    ///
    /// * `time` - The time point at which to evaluate the survival probability
    ///
    /// # Returns
    ///
    /// The survival probability at the specified time. Returns `1.0` if the time
    /// is before the first event, or the last known survival probability if the
    /// time is after the last event.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oxidris_stats::survival::KaplanMeierCurve;
    /// let data = vec![(10, false), (20, false)];
    /// let curve = KaplanMeierCurve::from_data(data);
    ///
    /// assert_eq!(curve.survival_at(5), 1.0);  // Before first event
    /// assert!(curve.survival_at(15) < 1.0);   // After first event
    /// ```
    #[must_use]
    pub fn survival_at(&self, time: usize) -> f64 {
        if self.times.is_empty() {
            return 1.0;
        }

        // Find the last time point <= target time
        for i in (0..self.times.len()).rev() {
            if self.times[i] <= time {
                return self.survival_prob[i];
            }
        }

        // Before first event, survival is 1.0
        1.0
    }
}
