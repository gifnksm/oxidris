use super::board_feature::ALL_BOARD_FEATURES_COUNT;
use rand::Rng;
use rand_distr::Normal;
use std::array;

#[derive(Debug, Clone)]
pub struct WeightSet<const N: usize>([f32; N]);

impl WeightSet<ALL_BOARD_FEATURES_COUNT> {
    pub const AGGRO: Self = WeightSet([
        0.327_024_2,     // Holes Penalty (x3.597)
        0.204_306_62,    // Hole Depth Penalty (x2.247)
        0.139_662_37,    // Row Transitions Penalty (x1.536)
        0.002_453_668_3, // Column Transitions Penalty (x0.027)
        0.029_498_829,   // Surface Roughness Penalty (x0.324)
        0.005_754_522_5, // Well Depth Penalty (x0.063)
        0.115_562_31,    // Deep Well Risk (x1.271)
        0.067_098_595,   // Top-Out Risk (x0.738)
        0.0,             // Total Height Penalty (x0.000)
        0.037_579_052,   // Lines Clear Bonus (x0.413)
        0.071_059_88,    // I-Well Reward (x0.782)
    ]);
    pub const DEFENSIVE: Self = WeightSet([
        0.102_967_23,    // Holes Penalty (x1.133)
        0.097_032_13,    // Hole Depth Penalty (x1.067)
        0.127_356_34,    // Row Transitions Penalty (x1.401)
        0.001_720_160_8, // Column Transitions Penalty (x0.019)
        0.082_752_49,    // Surface Roughness Penalty (x0.910)
        0.140_488_31,    // Well Depth Penalty (x1.545)
        0.094_674_75,    // Deep Well Risk (x1.041)
        0.133_106_14,    // Top-Out Risk (x1.464)
        0.192_896_2,     // Total Height Penalty (x2.122)
        0.025_241_91,    // Lines Clear Bonus (x0.278)
        0.001_764_394_6, // I-Well Reward (x0.019)
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
