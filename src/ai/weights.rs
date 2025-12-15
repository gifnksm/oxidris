use super::metrics::METRIC_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub(crate) struct WeightSet<const N: usize>(pub(crate) [f32; N]);

impl WeightSet<{ METRIC_COUNT }> {
    pub(crate) const BEST: Self = WeightSet([
        0.341_591_36,
        0.448_022_6,
        0.278_297_16,
        0.017_819_492,
        0.311_993_15,
        0.161_378_46,
        0.196_122_18,
        0.076_095_2,
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
}
