use super::board_feature::ALL_BOARD_FEATURES_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_BOARD_FEATURES_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.338_456_33,    // Holes Penalty (x3.723)
        0.157_514_24,    // Hole Depth Penalty (x1.733)
        0.110_240_1,     // Row Transitions Penalty (x1.213)
        0.016_150_504,   // Column Transitions Penalty (x0.178)
        0.017_910_853,   // Surface Roughness Penalty (x0.197)
        0.006_789_481_3, // Well Depth Penalty (x0.075)
        0.156_199_77,    // Deep Well Risk (x1.718)
        0.082_701_12,    // Top-Out Risk (x0.910)
        0.000_330_350_1, // Total Height Penalty (x0.004)
        0.049_564_66,    // Lines Clear Bonus (x0.545)
        0.064_142_43,    // I-Well Reward (x0.706)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.183_131_95,  // Holes Penalty (x2.014)
        0.108_931_2,   // Hole Depth Penalty (x1.198)
        0.262_073_13,  // Row Transitions Penalty (x2.883)
        0.014_965_9,   // Column Transitions Penalty (x0.165)
        0.069_496_44,  // Surface Roughness Penalty (x0.764)
        0.031_565_06,  // Well Depth Penalty (x0.347)
        0.056_278_78,  // Deep Well Risk (x0.619)
        0.085_730_44,  // Top-Out Risk (x0.943)
        0.127_369_23,  // Total Height Penalty (x1.401)
        0.045_144_32,  // Lines Clear Bonus (x0.497)
        0.015_313_635, // I-Well Reward (x0.168)
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
