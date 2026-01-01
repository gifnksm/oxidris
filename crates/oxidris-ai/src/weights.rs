use rand::Rng;
use rand_distr::Normal;

pub fn from_fn<F>(mut f: F, len: usize) -> Vec<f32>
where
    F: FnMut(usize) -> f32,
{
    let mut values = Vec::with_capacity(len);
    for i in 0..len {
        values.push(f(i));
    }
    values
}

pub(crate) fn random<R>(rng: &mut R, max_weight: f32, len: usize) -> Vec<f32>
where
    R: Rng + ?Sized,
{
    from_fn(|_| rng.random_range(0.0..=max_weight), len)
}

pub(crate) fn blx_alpha<R>(
    p1: &[f32],
    p2: &[f32],
    alpha: f32,
    max_weight: f32,
    rng: &mut R,
) -> Vec<f32>
where
    R: Rng + ?Sized,
{
    assert_eq!(p1.len(), p2.len());
    from_fn(
        |i| {
            let x1 = p1[i];
            let x2 = p2[i];
            let min = f32::min(x1, x2);
            let max = f32::max(x1, x2);
            let d = max - min;
            let lower = min - alpha * d;
            let upper = max + alpha * d;
            rng.random_range(lower..=upper).clamp(0.0, max_weight)
        },
        p1.len(),
    )
}

pub(crate) fn mutate<R>(weights: &mut [f32], sigma: f32, max_weight: f32, rate: f32, rng: &mut R)
where
    R: Rng + ?Sized,
{
    let normal = Normal::new(0.0, sigma).unwrap();
    for w in weights {
        if rng.random_bool(rate.into()) {
            *w = (*w + rng.sample(normal)).clamp(0.0, max_weight);
        }
    }
}

pub(crate) fn normalize_l1(weights: &mut [f32]) {
    let sum: f32 = weights.iter().copied().sum();
    if sum > 0.0 {
        for w in weights {
            *w /= sum;
        }
    }
}
