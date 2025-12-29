use super::metrics::ALL_METRICS_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_METRICS_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.450_044_54,    // Holes Penalty (x4.500)
        0.133_644,       // Row Transitions Penalty (x1.336)
        0.002_653_806_5, // Column Transitions Penalty (x0.027)
        0.028_636_364,   // Surface Roughness Penalty (x0.286)
        0.019_406_16,    // Well Depth Penalty (x0.194)
        0.127_770_99,    // Deep Well Risk (x1.278)
        0.102_169_1,     // Top-Out Risk (x1.022)
        0.003_343_727_4, // Total Height Penalty (x0.033)
        0.062_952_705,   // Lines Clear Bonus (x0.630)
        0.069_378_53,    // I-Well Reward (x0.694)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.109_653,       // Holes Penalty (x1.097)
        0.153_128_22,    // Row Transitions Penalty (x1.531)
        0.004_127_479,   // Column Transitions Penalty (x0.041)
        0.072_307_974,   // Surface Roughness Penalty (x0.723)
        0.285_440_8,     // Well Depth Penalty (x2.854)
        0.054_085_21,    // Deep Well Risk (x0.541)
        0.076_833_904,   // Top-Out Risk (x0.768)
        0.155_718_33,    // Total Height Penalty (x1.557)
        0.084_585_75,    // Lines Clear Bonus (x0.846)
        0.004_119_361_8, // I-Well Reward (x0.041)
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
