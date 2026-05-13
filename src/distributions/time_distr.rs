use crate::errors::SamplerError;
use rand::Rng;
use rand_distr::{Distribution, Gamma};

///
/// Duration time sampler based on a gamma distribution.
///
#[derive(Debug)]
pub struct DurationSampler {
    gamma: Gamma<f64>,
}

impl DurationSampler {
    ///
    /// Create a new duration time sampler.
    ///
    pub fn new(avg_duration_s: f64, shape: f64) -> Result<Self, SamplerError> {
        let scale = avg_duration_s / shape;
        let gamma =
            Gamma::new(shape, scale).map_err(|e| SamplerError::InvalidParameter(e.to_string()))?;
        Ok(Self { gamma })
    }

    ///
    /// Sample a random duration from a gamma distribution.
    ///
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        self.gamma.sample(rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_duration_sampler() {
        let mut rng = StdRng::seed_from_u64(42);
        let sampler = DurationSampler::new(10.0, 0.5).unwrap();
        assert_approx_eq!(sampler.sample(&mut rng), 7.298816806468081);
    }

    #[test]
    fn test_duration_sampler_error() {
        let sampler = DurationSampler::new(10.0, -0.5).unwrap_err();
        assert!(matches!(sampler, SamplerError::InvalidParameter(_)));
    }
}
