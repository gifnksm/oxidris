use super::metrics::ALL_METRICS_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_METRICS_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.414_283_1,   // Covered Holes (x3.729)
        0.181_405_57,  // Row Transitions (x1.633)
        0.009_915_61,  // Column Transitions (x0.089)
        0.045_901_358, // Surface Roughness (x0.413)
        0.126_208_47,  // Max Height (x1.136)
        0.004_029_349, // Deep Well Risk (x0.036)
        0.009_261_049, // Sum of Heights (x0.083)
        0.093_007_59,  // Lines Clear Reward (x0.837)
        0.115_987_94,  // I-Well Reward (x1.044)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.240_808_22,  // Covered Holes (x2.167)
        0.203_893_11,  // Row Transitions (x1.835)
        0.028_583_506, // Column Transitions (x0.257)
        0.037_466_74,  // Surface Roughness (x0.337)
        0.031_979_535, // Max Height (x0.288)
        0.215_694_52,  // Deep Well Risk (x1.941)
        0.059_432_928, // Sum of Heights (x0.535)
        0.169_650_54,  // Lines Clear Reward (x1.527)
        0.012_490_933, // I-Well Reward (x0.112)
    ]);
}

impl<const N: usize> WeightSet<N> {
    pub(crate) const fn from_array(arr: [f32; N]) -> Self {
        Self(arr)
    }

    pub(crate) fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> f32,
    {
        Self::from_array(array::from_fn(f))
    }

    pub(crate) const fn as_array(&self) -> [f32; N] {
        self.0
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
        let p1 = p1.as_array();
        let p2 = p2.as_array();
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
        let mut weights = self.as_array();
        let normal = Normal::new(0.0, sigma).unwrap();
        for w in &mut weights {
            if rng.random_bool(rate) {
                *w = (*w + rng.sample(normal)).clamp(0.0, max_w);
            }
        }
        *self = Self::from_array(weights);
    }

    pub(crate) fn normalize_l1(&mut self) {
        let mut weights = self.as_array();
        let sum: f32 = weights.into_iter().sum();
        if sum > 0.0 {
            for w in &mut weights {
                *w /= sum;
            }
        }
        *self = Self::from_array(weights);
    }
}
