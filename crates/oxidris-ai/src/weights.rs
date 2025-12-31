use super::board_feature::ALL_BOARD_FEATURES_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_BOARD_FEATURES_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.429_029_4,      // Holes Penalty (x5.148)
        0.066_070_01,     // Hole Depth Penalty (x0.793)
        0.093_926_28,     // Row Transitions Penalty (x1.127)
        0.000_256_118_86, // Column Transitions Penalty (x0.003)
        0.050_718_028,    // Surface Bumpiness Penalty (x0.609)
        0.021_752_236,    // Surface Roughness Penalty (x0.261)
        0.018_839_05,     // Well Depth Penalty (x0.226)
        0.055_612_948,    // Deep Well Risk (x0.667)
        0.121_907_71,     // Top-Out Risk (x1.463)
        0.0,              // Total Height Penalty (x0.000)
        0.059_491_25,     // Lines Clear Bonus (x0.714)
        0.082_396_984,    // I-Well Reward (x0.989)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.227_313_28,  // Holes Penalty (x2.728)
        0.111_004_815, // Hole Depth Penalty (x1.332)
        0.032_333_97,  // Row Transitions Penalty (x0.388)
        0.010_147_94,  // Column Transitions Penalty (x0.122)
        0.091_924_7,   // Surface Bumpiness Penalty (x1.103)
        0.062_576_4,   // Surface Roughness Penalty (x0.751)
        0.147_894_28,  // Well Depth Penalty (x1.775)
        0.015_304_379, // Deep Well Risk (x0.184)
        0.116_314_37,  // Top-Out Risk (x1.396)
        0.142_704_46,  // Total Height Penalty (x1.712)
        0.028_222_857, // Lines Clear Bonus (x0.339)
        0.014_258_568, // I-Well Reward (x0.171)
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
