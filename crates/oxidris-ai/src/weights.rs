use crate::metrics::MetricValues;

use super::metrics::METRIC_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet(MetricValues);

impl WeightSet {
    pub const AGGRO: Self = WeightSet(MetricValues {
        covered_holes: 0.434_629_8,
        row_transitions: 0.086_059_77,
        column_transitions: 0.098_730_02,
        surface_roughness: 0.004_102_067_6,
        height_risk: 0.138_786_9,
        deep_well_risk: 0.216_158_88,
        sum_of_heights: 0.0,
        lines_cleared: 0.008_848_529,
        i_well_reward: 0.012_684_016,
    });
    pub const DEFENSIVE: Self = WeightSet(MetricValues {
        covered_holes: 0.107_194_17,
        row_transitions: 0.234_455_47,
        column_transitions: 0.130_683_21,
        surface_roughness: 0.022_107_849,
        height_risk: 0.071_108_915,
        deep_well_risk: 0.105_950_765,
        sum_of_heights: 0.158_108_67,
        lines_cleared: 0.154_509_29,
        i_well_reward: 0.015_881_691,
    });
}

impl WeightSet {
    pub(crate) const fn from_array(arr: [f32; METRIC_COUNT]) -> Self {
        Self(MetricValues::from_array(arr))
    }

    pub(crate) fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> f32,
    {
        Self::from_array(array::from_fn(f))
    }

    pub(crate) const fn to_array(&self) -> [f32; METRIC_COUNT] {
        self.0.to_array()
    }

    pub(crate) fn random<R>(rng: &mut R, max_w: f32) -> Self
    where
        R: Rng + ?Sized,
    {
        Self::from_fn(|_| rng.random_range(0.0..=max_w))
    }

    pub(crate) fn blx_alpha<R>(p1: &Self, p2: &Self, alpha: f32, max_w: f32, rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        let p1 = p1.to_array();
        let p2 = p2.to_array();
        Self::from_fn(|i| {
            let x1 = p1[i];
            let x2 = p2[i];
            let min = f32::min(x1, x2);
            let max = f32::max(x1, x2);
            let d = max - min;
            let lower = min - alpha * d;
            let upper = max + alpha * d;
            rng.random_range(lower..=upper).clamp(0.0, max_w)
        })
    }

    pub(crate) fn mutate<R>(&mut self, sigma: f32, max_w: f32, rate: f64, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let mut weights = self.to_array();
        let normal = Normal::new(0.0, sigma).unwrap();
        for w in &mut weights {
            if rng.random_bool(rate) {
                *w = (*w + rng.sample(normal)).clamp(0.0, max_w);
            }
        }
        *self = Self::from_array(weights);
    }

    pub(crate) fn normalize_l1(&mut self) {
        let mut weights = self.to_array();
        let sum: f32 = weights.into_iter().sum();
        if sum > 0.0 {
            for w in &mut weights {
                *w /= sum;
            }
        }
        *self = Self::from_array(weights);
    }
}
