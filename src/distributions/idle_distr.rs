use rand::Rng;
use rand_distr::{Distribution, Gamma};

///
/// Sample a random vehicle idle time from a gamma distribution
///
pub fn sample_idle_time(avg_idle_duration_s: f64, shape: f64, rng: &mut dyn Rng) -> f64 {
    let scale = avg_idle_duration_s / shape;
    let gamma = Gamma::new(shape, scale).unwrap();

    gamma.sample(rng)
}
