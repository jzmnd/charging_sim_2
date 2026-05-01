use rand::{Rng, RngExt};
use rand_distr::weighted::WeightedIndex;
use rand_distr::{Distribution, Poisson};

#[derive(Debug)]
pub struct ArrivalSampler {
    hour_weights: [f64; 24],
    day_weights: [f64; 7],
    avg_sessions_per_day: f64,
}

impl ArrivalSampler {
    ///
    /// Create a new arrival time sampler.
    ///
    pub fn new(hour_weights: [f64; 24], day_weights: [f64; 7], avg_sessions_per_day: f64) -> Self {
        Self {
            hour_weights,
            day_weights,
            avg_sessions_per_day,
        }
    }

    ///
    /// Sample arrival times (in seconds) for a given number of days.
    /// Assumes the first day is the first weights in the `day_weights` array.
    ///
    pub fn sample_arrivals(&self, num_days: u64, rng: &mut dyn Rng) -> Vec<u64> {
        let hour_dist = WeightedIndex::new(self.hour_weights).unwrap();
        let day_weights_sum: f64 = self.day_weights.iter().sum();
        let day_weights_scale = if day_weights_sum > 0.0 {
            7.0 / day_weights_sum
        } else {
            1.0
        };
        let mut arrivals = Vec::new();

        for day in 0..num_days {
            let weekday = (day % 7) as usize;
            let day_weight = self.day_weights[weekday] * day_weights_scale;

            let poisson = Poisson::new(self.avg_sessions_per_day * day_weight).unwrap();
            let sessions = poisson.sample(rng) as u64;

            for _ in 0..sessions {
                let hour = hour_dist.sample(rng) as u64;
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
