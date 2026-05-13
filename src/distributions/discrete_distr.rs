use log::warn;
use rand::Rng;
use rand::prelude::IndexedRandom;
use rand_distr::Distribution;
use rand_distr::weighted::WeightedIndex;

///
/// Discrete item sampler.
///
#[derive(Debug)]
pub struct ItemSampler<T> {
    items: Vec<T>,
    weighted_idx: Option<WeightedIndex<f64>>,
}

impl<T: Copy> ItemSampler<T> {
    ///
    /// Create a new item sampler.
    ///
    pub fn new(items: &[T]) -> Self {
        Self {
            items: items.to_vec(),
            weighted_idx: None,
        }
    }

    ///
    /// Set item weights.
    ///
    pub fn set_weights(&mut self, weights: &[f64]) {
        if weights.len() != self.items.len() {
            warn!("Weights must be the same length as the number of items");
            return;
        }
        match WeightedIndex::new(weights) {
            Ok(weighted_idx) => self.weighted_idx = Some(weighted_idx),
            Err(e) => warn!("Invalid weights: {}", e),
        }
    }

    ///
    /// Sample an item from the list of items.
    /// Uses the configured weights if set, otherwise samples uniformly.
    ///
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<T> {
        if self.items.is_empty() {
            return None;
        }
        match &self.weighted_idx {
            Some(weighted_idx) => {
                let idx = weighted_idx.sample(rng);
                Some(self.items[idx])
            }
            // fall back to uniform distribution
            None => self.items.choose(rng).copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_item_sampler() {
        let mut rng = StdRng::seed_from_u64(42);
        let sampler = ItemSampler::new(&[1, 2, 3]);
        let item = sampler.sample(&mut rng).unwrap();
        assert_eq!(item, 1);
    }

    #[test]
    fn test_item_sampler_weighted() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut sampler = ItemSampler::new(&[1, 2, 3]);
        sampler.set_weights(&[0.0, 0.0, 1.0]);
        let item = sampler.sample(&mut rng).unwrap();
        assert_eq!(item, 3);
    }

    #[test]
    fn test_item_sampler_no_items() {
        let mut rng = StdRng::seed_from_u64(42);
        let sampler: ItemSampler<f64> = ItemSampler::new(&[]);
        assert!(sampler.sample(&mut rng).is_none())
    }
}
