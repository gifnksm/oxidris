use super::board_feature::ALL_BOARD_FEATURES_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_BOARD_FEATURES_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.335_840_02,    // Holes Penalty (x3.694)
        0.154_844_2,     // Hole Depth Penalty (x1.703)
        0.100_580_48,    // Row Transitions Penalty (x1.106)
        0.013_257_7,     // Column Transitions Penalty (x0.146)
        0.017_089_589,   // Surface Roughness Penalty (x0.188)
        0.001_605_333_4, // Well Depth Penalty (x0.018)
        0.055_428_28,    // Deep Well Risk (x0.610)
        0.190_222_22,    // Top-Out Risk (x2.092)
        0.000_687_385_3, // Total Height Penalty (x0.008)
        0.066_430_375,   // Lines Clear Bonus (x0.731)
        0.064_014_465,   // I-Well Reward (x0.704)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.126_233_62,  // Holes Penalty (x1.389)
        0.147_662_85,  // Hole Depth Penalty (x1.624)
        0.127_291_6,   // Row Transitions Penalty (x1.400)
        0.016_144_527, // Column Transitions Penalty (x0.178)
        0.025_313_282, // Surface Roughness Penalty (x0.278)
        0.075_383_67,  // Well Depth Penalty (x0.829)
        0.059_969_07,  // Deep Well Risk (x0.660)
        0.162_240_28,  // Top-Out Risk (x1.785)
        0.138_716_34,  // Total Height Penalty (x1.526)
        0.106_450_39,  // Lines Clear Bonus (x1.171)
        0.014_594_393, // I-Well Reward (x0.161)
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
