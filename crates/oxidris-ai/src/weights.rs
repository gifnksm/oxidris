use super::metrics::ALL_METRICS_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_METRICS_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.429_964_54,     // Holes Penalty (x3.870)
        0.201_704_71,     // Row Transitions Penalty (x1.815)
        0.000_804_378_36, // Column Transitions Penalty (x0.007)
        0.052_598_39,     // Surface Roughness Penalty (x0.473)
        0.008_747_218,    // Well Depth Penalty (x0.079)
        0.125_386_98,     // Top-Out Risk (x1.128)
        0.016_567_592,    // Total Height Penalty (x0.149)
        0.058_255_833,    // Lines Clear Bonus (x0.524)
        0.105_970_39,     // I-Well Reward (x0.954)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.122_703_776,   // Holes Penalty (x1.104)
        0.242_151_08,    // Row Transitions Penalty (x2.179)
        0.0,             // Column Transitions Penalty (x0.000)
        0.006_782_503_3, // Surface Roughness Penalty (x0.061)
        0.272_973_5,     // Well Depth Penalty (x2.457)
        0.110_206_21,    // Top-Out Risk (x0.992)
        0.093_641_36,    // Total Height Penalty (x0.843)
        0.130_498_62,    // Lines Clear Bonus (x1.174)
        0.021_042_93,    // I-Well Reward (x0.189)
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
