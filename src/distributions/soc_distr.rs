use rand::Rng;
use rand_distr::{Beta, Distribution};

fn beta_from_mean_kappa(mean: f64, kappa: f64) -> Beta<f64> {
    let alpha = mean * kappa;
    let beta = (1.0 - mean) * kappa;

    Beta::new(alpha, beta).unwrap()
}

///
/// Sample a random starting and target SOC from two beta distributions.
/// Resamples to ensure that the targe SOC is always greater than the
/// starting SOC.
///
pub fn sample_socs(
    avg_start_soc: f64,
    avg_target_soc: f64,
    kappa_start: f64,
    kappa_target: f64,
    rng: &mut dyn Rng,
) -> (f64, f64) {
    let beta_start = beta_from_mean_kappa(avg_start_soc, kappa_start);
    let beta_end = beta_from_mean_kappa(avg_target_soc, kappa_target);

    let start_soc = beta_start.sample(rng);

    loop {
        let target_soc = beta_end.sample(rng);
        if target_soc > start_soc {
            return (start_soc, target_soc);
        }
    }
}
