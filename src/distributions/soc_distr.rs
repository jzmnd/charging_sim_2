use crate::errors::SamplerError;
use rand::Rng;
use rand_distr::{Beta, Distribution};

#[derive(Debug)]
pub struct SocSampler {
    beta_start: Beta<f64>,
    beta_target: Beta<f64>,
}

impl SocSampler {
    ///
    /// Create a new SOC sampler.
    ///
    pub fn new(
        avg_start_soc: f64,
        avg_target_soc: f64,
        kappa_start: f64,
        kappa_target: f64,
    ) -> Result<Self, SamplerError> {
        Ok(Self {
            beta_start: Self::beta_from_mean_kappa(avg_start_soc, kappa_start)?,
            beta_target: Self::beta_from_mean_kappa(avg_target_soc, kappa_target)?,
        })
    }

    ///
    /// Get a Beta distribution given a mean and kappa.
    ///
    fn beta_from_mean_kappa(mean: f64, kappa: f64) -> Result<Beta<f64>, SamplerError> {
        let alpha = mean * kappa;
        let beta = (1.0 - mean) * kappa;

        Beta::new(alpha, beta).map_err(|e| SamplerError::InvalidParameter(e.to_string()))
    }

    ///
    /// Sample a random starting and target SOC from two beta distributions.
    /// Resamples to ensure that the target SOC is always greater than the
    /// starting SOC.
    ///
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> (f64, f64) {
        let start_soc = self.beta_start.sample(rng);

        loop {
            let target_soc = self.beta_target.sample(rng);
            if target_soc > start_soc {
                return (start_soc, target_soc);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_soc_sampler() {
        let mut rng = StdRng::seed_from_u64(42);
        let sampler = SocSampler::new(0.2, 0.8, 9.0, 12.0).unwrap();
        let (start_soc, end_soc) = sampler.sample(&mut rng);
        assert_approx_eq!(start_soc, 0.21116744878758403);
        assert_approx_eq!(end_soc, 0.7470459768047447);
    }

    #[test]
    fn test_soc_sampler_close() {
        let mut rng = StdRng::seed_from_u64(42);
        let sampler = SocSampler::new(0.45, 0.45, 9.0, 12.0).unwrap();
        let (start_soc, end_soc) = sampler.sample(&mut rng);
        assert_approx_eq!(start_soc, 0.46251520872968643);
        assert_approx_eq!(end_soc, 0.5073381792715856);
    }

    #[test]
    fn test_soc_sampler_error() {
        let sampler = SocSampler::new(0.2, 0.8, -9.0, 12.0).unwrap_err();
        assert!(matches!(sampler, SamplerError::InvalidParameter(_)));
    }
}
