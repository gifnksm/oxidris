use super::metrics::ALL_METRICS_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_METRICS_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.407_965_3,   // Holes Penalty (x3.672)
        0.212_051_47,  // Row Transitions Penalty (x1.908)
        0.014_619_024, // Column Transitions Penalty (x0.132)
        0.028_094_463, // Surface Roughness Penalty (x0.253)
        0.007_172_004, // Well Depth Penalty (x0.065)
        0.113_304_056, // Top-Out Risk (x1.020)
        0.0,           // Total Height Penalty (x0.000)
        0.086_369_514, // Lines Clear Bonus (x0.777)
        0.130_424_22,  // I-Well Reward (x1.174)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.171_370_98,  // Holes Penalty (x1.542)
        0.214_435_43,  // Row Transitions Penalty (x1.930)
        0.014_529_546, // Column Transitions Penalty (x0.131)
        0.036_790_52,  // Surface Roughness Penalty (x0.331)
        0.178_688_81,  // Well Depth Penalty (x1.608)
        0.027_978_083, // Top-Out Risk (x0.252)
        0.145_202_47,  // Total Height Penalty (x1.307)
        0.183_667_21,  // Lines Clear Bonus (x1.653)
        0.027_336_968, // I-Well Reward (x0.246)
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
