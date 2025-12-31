use super::board_feature::ALL_BOARD_FEATURES_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_BOARD_FEATURES_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.373_829_07,    // Holes Penalty (x5.234)
        0.080_027_12,    // Hole Depth Penalty (x1.120)
        0.086_602_025,   // Row Transitions Penalty (x1.212)
        0.006_008_607_3, // Column Transitions Penalty (x0.084)
        0.052_111_614,   // Surface Bumpiness Penalty (x0.730)
        0.036_412_79,    // Surface Roughness Penalty (x0.510)
        0.008_209_276,   // Well Depth Penalty (x0.115)
        0.097_872_89,    // Deep Well Risk (x1.370)
        0.015_906_55,    // Max Height Penalty (x0.223)
        0.004_509_309_3, // Center Columns Penalty (x0.063)
        0.101_194_866,   // Top-Out Risk (x1.417)
        0.016_144_508,   // Total Height Penalty (x0.226)
        0.037_760_97,    // Lines Clear Bonus (x0.529)
        0.083_410_405,   // I-Well Reward (x1.168)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.194_811_79,  // Holes Penalty (x2.727)
        0.080_317_44,  // Hole Depth Penalty (x1.124)
        0.122_056_16,  // Row Transitions Penalty (x1.709)
        0.028_538_82,  // Column Transitions Penalty (x0.400)
        0.016_977_979, // Surface Bumpiness Penalty (x0.238)
        0.018_851_45,  // Surface Roughness Penalty (x0.264)
        0.063_216_31,  // Well Depth Penalty (x0.885)
        0.148_819_07,  // Deep Well Risk (x2.083)
        0.104_424_49,  // Max Height Penalty (x1.462)
        0.042_676_45,  // Center Columns Penalty (x0.597)
        0.025_291_076, // Top-Out Risk (x0.354)
        0.046_873_4,   // Total Height Penalty (x0.656)
        0.091_819_815, // Lines Clear Bonus (x1.285)
        0.015_325_842, // I-Well Reward (x0.215)
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
