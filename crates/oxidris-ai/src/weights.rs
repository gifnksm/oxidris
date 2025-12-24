use super::metrics::METRIC_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::{array, fmt};

#[derive(Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl<const N: usize> fmt::Debug for WeightSet<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl WeightSet<{ METRIC_COUNT }> {
    pub const AGGRO: Self = WeightSet([
        0.365_295_86,
        0.159_927_16,
        0.284_307_42,
        0.002_541_174_9,
        0.136_750_53,
        0.019_371_7,
        0.001_287_397_7,
        0.006_894_496_7,
        0.023_624_308,
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.227_464_81,
        0.195_648_88,
        0.255_317_93,
        0.025_598_804,
        0.052_447_02,
        0.072_872_87,
        0.084_215_49,
        0.067_219_265,
        0.019_214_835,
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

    pub(crate) const fn to_array(&self) -> [f32; N] {
        self.0
    }

    pub(crate) fn random<R>(rng: &mut R, max_w: f32) -> Self
    where
        R: Rng + ?Sized,
    {
        Self(array::from_fn(|_| rng.random_range(0.0..=max_w)))
    }

    pub(crate) fn blx_alpha<R>(p1: &Self, p2: &Self, alpha: f32, max_w: f32, rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        let p1 = p1.to_array();
        let p2 = p2.to_array();
        Self(array::from_fn(|i| {
            let x1 = p1[i];
            let x2 = p2[i];
            let min = f32::min(x1, x2);
            let max = f32::max(x1, x2);
            let d = max - min;
            let lower = min - alpha * d;
            let upper = max + alpha * d;
            rng.random_range(lower..=upper).clamp(0.0, max_w)
        }))
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
        *self = Self(weights);
    }

    pub(crate) fn normalize_l1(&mut self) {
        let mut weights = self.to_array();
        let sum: f32 = weights.into_iter().sum();
        if sum > 0.0 {
            for w in &mut weights {
                *w /= sum;
            }
        }
        *self = Self(weights);
    }
}
