use super::metrics::METRIC_COUNT;
use rand::{Rng, distr::StandardUniform, prelude::Distribution};
use std::fmt;

#[derive(Clone, Copy)]
pub(crate) struct Weight(pub(crate) f32);

impl fmt::Debug for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2e}", self.0)
    }
}

impl Distribution<Weight> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Weight {
        Weight(rng.random_range(0.0..0.5))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WeightSet<const N: usize>(pub(crate) [Weight; N]);

impl Default for WeightSet<{ METRIC_COUNT }> {
    fn default() -> Self {
        Self([
            Weight(0.149_261_53),
            Weight(0.135_986_27),
            Weight(0.746_309_46),
            Weight(0.556_209_2),
        ])
    }
}

impl<const N: usize> Distribution<WeightSet<N>> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> WeightSet<N> {
        WeightSet(rng.random())
    }
}

impl<const N: usize> WeightSet<N> {
    pub(crate) fn two_point_crossover<R>(parent1: &Self, parent2: &Self, rng: &mut R) -> [Self; 2]
    where
        R: Rng + ?Sized,
    {
        let mut child1 = parent1.0;
        let mut child2 = parent2.0;
        let p1 = rng.random_range(0..N);
        let p2 = rng.random_range(0..N);
        let (start, end) = if p1 < p2 { (p1, p2) } else { (p2, p1) };
        child1[start..end].copy_from_slice(&parent2.0[start..end]);
        child2[start..end].copy_from_slice(&parent1.0[start..end]);
        [Self(child1), Self(child2)]
    }

    pub(crate) fn mutation<R>(&self, rng: &mut R, mutation_rate: f64) -> Self
    where
        R: Rng + ?Sized,
    {
        Self(self.0.map(|weight| {
            if rng.random_bool(mutation_rate) {
                rng.random()
            } else {
                weight
            }
        }))
    }
}
