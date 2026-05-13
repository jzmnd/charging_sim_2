use crate::errors::SamplerError;
use rand::{Rng, RngExt};
use rand_distr::weighted::WeightedIndex;
use rand_distr::{Distribution, Poisson};

///
/// Sampler for vehicle arrival times.
///
#[derive(Debug)]
pub struct ArrivalSampler {
    hour_dist: WeightedIndex<f64>,
    day_poissons: [Poisson<f64>; 7],
}

impl ArrivalSampler {
    ///
    /// Create a new arrival time sampler.
    ///
    pub fn new(
        hour_weights: [f64; 24],
        day_weights: [f64; 7],
        avg_sessions_per_day: f64,
    ) -> Result<Self, SamplerError> {
        let hour_dist = WeightedIndex::new(hour_weights)
            .map_err(|e| SamplerError::InvalidParameter(e.to_string()))?;

        let day_weights_sum: f64 = day_weights.iter().sum();
        let day_weights_scale = if day_weights_sum > 0.0 {
            7.0 / day_weights_sum
        } else {
            1.0
        };

        let rates: [f64; 7] =
            std::array::from_fn(|i| avg_sessions_per_day * day_weights[i] * day_weights_scale);
        for rate in rates {
            Poisson::new(rate).map_err(|e| SamplerError::InvalidParameter(e.to_string()))?;
        }

        Ok(Self {
            hour_dist,
            day_poissons: rates.map(|rate| Poisson::new(rate).unwrap()),
        })
    }

    ///
    /// Sample arrival times (in seconds) for a given number of days.
    /// Assumes the first day is the first weights in the `day_weights` array.
    ///
    pub fn sample<R: Rng + ?Sized>(&self, num_days: u64, rng: &mut R) -> Vec<u64> {
        let mut arrivals = Vec::new();

        for day in 0..num_days {
            let weekday = (day % 7) as usize;
            let sessions = self.day_poissons[weekday].sample(rng) as u64;

            for _ in 0..sessions {
                let hour = self.hour_dist.sample(rng) as u64;
                let minute = rng.random_range(0..60);
                let second = rng.random_range(0..60);
                let total_seconds = day * 86400 + hour * 3600 + minute * 60 + second;

                arrivals.push(total_seconds);
            }
        }

        arrivals.sort_unstable();
        arrivals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_arrival_sampler() {
        let mut rng = StdRng::seed_from_u64(42);
        let hour_weights = [
            0.0, 0.0, 0.05, 0.05, 0.1, 0.1, 0.15, 0.25, 0.3, 0.35, 0.4, 0.4, 0.35, 0.35, 0.4, 0.45,
            0.5, 0.55, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1,
        ];
        let day_weights = [0.95, 0.8, 0.8, 0.75, 0.8, 0.85, 1.0];
        let sampler = ArrivalSampler::new(hour_weights, day_weights, 100.0).unwrap();
        let arrival_times = sampler.sample(2, &mut rng);
        assert_eq!(arrival_times.len(), 220);
        assert!(*arrival_times.iter().max().unwrap() < 2 * 86400);
    }

    #[test]
    fn test_arrival_sampler_error() {
        let hour_weights = [
            0.0, 0.0, 0.05, 0.05, 0.1, 0.1, 0.15, 0.25, 0.3, 0.35, 0.4, 0.4, 0.35, 0.35, 0.4, 0.45,
            0.5, 0.55, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1,
        ];
        let day_weights = [0.95, 0.8, 0.8, 0.75, 0.8, 0.85, 1.0];
        let sampler = ArrivalSampler::new(hour_weights, day_weights, -100.0).unwrap_err();
        assert!(matches!(sampler, SamplerError::InvalidParameter(_)));
    }
}
