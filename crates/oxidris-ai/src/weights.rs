use super::board_feature::ALL_BOARD_FEATURES_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_BOARD_FEATURES_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.363_839_66,    // Holes Penalty (x4.002)
        0.081_490_03,    // Hole Depth Penalty (x0.896)
        0.148_351_91,    // Row Transitions Penalty (x1.632)
        0.000_305_067_8, // Column Transitions Penalty (x0.003)
        0.023_110_254,   // Surface Roughness Penalty (x0.254)
        0.014_985_618,   // Well Depth Penalty (x0.165)
        0.165_595_6,     // Deep Well Risk (x1.822)
        0.068_250_14,    // Top-Out Risk (x0.751)
        0.001_151_630_1, // Total Height Penalty (x0.013)
        0.063_833_51,    // Lines Clear Bonus (x0.702)
        0.069_086_61,    // I-Well Reward (x0.760)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.260_091_48,    // Holes Penalty (x2.861)
        0.028_879_97,    // Hole Depth Penalty (x0.318)
        0.092_790_57,    // Row Transitions Penalty (x1.021)
        0.0,             // Column Transitions Penalty (x0.000)
        0.052_558_94,    // Surface Roughness Penalty (x0.578)
        0.197_318_23,    // Well Depth Penalty (x2.171)
        0.005_194_588_6, // Deep Well Risk (x0.057)
        0.047_756_79,    // Top-Out Risk (x0.525)
        0.186_998_62,    // Total Height Penalty (x2.057)
        0.094_327_74,    // Lines Clear Bonus (x1.038)
        0.034_083_083,   // I-Well Reward (x0.375)
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

    #[must_use]
    pub const fn as_array(&self) -> [f32; N] {
        self.0
    }

    pub(crate) fn random<R>(rng: &mut R, max_weight: f32) -> Self
    where
        R: Rng + ?Sized,
    {
        Self::from_fn(|_| rng.random_range(0.0..=max_weight))
    }

    pub(crate) fn blx_alpha<R>(
        p1: &Self,
        p2: &Self,
        alpha: f32,
        max_weight: f32,
        rng: &mut R,
    ) -> Self
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
            rng.random_range(lower..=upper).clamp(0.0, max_weight)
        })
    }

    pub(crate) fn mutate<R>(&mut self, sigma: f32, max_weight: f32, rate: f32, rng: &mut R)
    where
        R: Rng + ?Sized,
    {
        let mut weights = self.as_array();
        let normal = Normal::new(0.0, sigma).unwrap();
        for w in &mut weights {
            if rng.random_bool(rate.into()) {
                *w = (*w + rng.sample(normal)).clamp(0.0, max_weight);
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
