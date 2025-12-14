use super::metrics::METRIC_COUNT;
use rand::{Rng, distr::StandardUniform, prelude::Distribution};

#[derive(Debug, Clone)]
pub(crate) struct WeightSet<const N: usize>(pub(crate) [u16; N]);

impl Default for WeightSet<{ METRIC_COUNT }> {
    fn default() -> Self {
        Self([40865, 559, 59765, 51848])
    }
}

impl<const N: usize> Distribution<WeightSet<N>> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> WeightSet<N> {
        WeightSet(rng.random())
    }
}
