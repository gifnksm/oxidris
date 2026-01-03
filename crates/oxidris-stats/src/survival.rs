/// Kaplan-Meier survival curve
#[derive(Debug, Clone)]
pub struct KaplanMeierCurve {
    /// Time points
    pub times: Vec<usize>,
    /// Survival probability at each time point
    pub survival_prob: Vec<f64>,
    /// Number at risk at each time point
    pub at_risk: Vec<usize>,
    /// Number of events at each time point
    pub events: Vec<usize>,
}

impl KaplanMeierCurve {
    /// Calculate Kaplan-Meier estimate from survival data
    /// data: `Vec<(time, is_censored)>`
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

    /// Get median survival time (time when survival probability drops to 50%)
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

    /// Get survival probability at a specific time
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
