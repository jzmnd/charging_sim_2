use rand::prelude::IndexedRandom;
use rand_distr::Distribution;
use rand_distr::weighted::WeightedIndex;
use uuid::Uuid;

///
/// Sample a charge profile ID from a list.
/// Optionally provide weights for sampling each ID.
///
pub fn sample_charge_profile(ids: &[Uuid], weights: Option<&[f64]>) -> Option<Uuid> {
    if ids.is_empty() {
        return None;
    }
    let mut rng = rand::rng();

    match weights {
        Some(w) if w.len() == ids.len() => {
            let dist = WeightedIndex::new(w).ok()?;
            let idx = dist.sample(&mut rng);
            Some(ids[idx])
        }
        // fall back to uniform distribution
        _ => ids.choose(&mut rng).copied(),
    }
}
