use super::metrics::ALL_METRICS_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_METRICS_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.511_994_54,    // Holes Penalty (x4.608)
        0.167_294_83,    // Row Transitions Penalty (x1.506)
        0.008_827_965,   // Column Transitions Penalty (x0.079)
        0.033_771_757,   // Surface Roughness Penalty (x0.304)
        0.012_187_055,   // Well Depth Penalty (x0.110)
        0.096_042_64,    // Top-Out Risk (x0.864)
        0.002_790_085_3, // Total Height Penalty (x0.025)
        0.084_101_796,   // Lines Clear Bonus (x0.757)
        0.082_989_35,    // I-Well Reward (x0.747)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.474_424_78,    // Holes Penalty (x4.270)
        0.171_952,       // Row Transitions Penalty (x1.548)
        0.0,             // Column Transitions Penalty (x0.000)
        0.028_104_998,   // Surface Roughness Penalty (x0.253)
        0.008_042_769,   // Well Depth Penalty (x0.072)
        0.191_902_79,    // Top-Out Risk (x1.727)
        0.004_426_616_7, // Total Height Penalty (x0.040)
        0.039_479_937,   // Lines Clear Bonus (x0.355)
        0.081_666_134,   // I-Well Reward (x0.735)
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
