use super::metrics::METRIC_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::{array, fmt};

#[derive(Clone)]
pub struct WeightSet<const N: usize>(pub(crate) [f32; N]);

impl<const N: usize> fmt::Debug for WeightSet<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl WeightSet<{ METRIC_COUNT }> {
    pub const AGGRO: Self = WeightSet([
        0.219_457_21,
        0.0,
        0.248_865_74,
        0.013_356_623,
        0.224_204_36,
        0.282_655_86,
        0.0,
        0.011_460_235,
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.223_957_17,
        0.136_435_08,
        0.220_598_61,
        0.006_791_787_7,
        0.100_103_706,
        0.070_308_48,
        0.106_088_705,
        0.135_716_48,
    ]);
}

impl<const N: usize> WeightSet<N> {
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
        Self(array::from_fn(|i| {
            let x1 = p1.0[i];
            let x2 = p2.0[i];
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
        let normal = Normal::new(0.0, sigma).unwrap();
        for w in &mut self.0 {
            if rng.random_bool(rate) {
                *w = (*w + rng.sample(normal)).clamp(0.0, max_w);
            }
        }
    }

    pub(crate) fn normalize_l1(&mut self) {
        let sum: f32 = self.0.iter().copied().sum();
        if sum > 0.0 {
            for w in &mut self.0 {
                *w /= sum;
            }
        }
    }
}
