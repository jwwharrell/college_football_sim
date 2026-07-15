//! RNG module for the college football simulator.
//!
//! This module provides a deterministic random number generator façade
//! to ensure reproducible simulation results.

use rand::distributions::{Distribution, Standard, Uniform, WeightedIndex};
use rand::prelude::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::ops::Range;

/// A deterministic random number generator façade.
///
/// This struct wraps a seeded ChaCha RNG to provide deterministic
/// random number generation for the simulator.
#[derive(Debug)]
pub struct SimRng {
    rng: ChaCha8Rng,
}

impl SimRng {
    /// Creates a new SimRng with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Returns a random boolean value.
    pub fn bool(&mut self) -> bool {
        self.rng.gen()
    }

    /// Returns a random integer in the range [0, n).
    pub fn int(&mut self, n: u32) -> u32 {
        self.rng.gen_range(0..n)
    }

    /// Returns a random integer in the given range.
    pub fn int_range<T>(&mut self, min: T, max: T) -> T
    where
        T: rand::distributions::uniform::SampleUniform + PartialOrd,
    {
        self.rng.gen_range(min..max)
    }

    /// Returns a random float in the range [0.0, 1.0).
    pub fn float(&mut self) -> f64 {
        self.rng.gen()
    }

    /// Returns a random float in the range [min, max).
    pub fn float_range(&mut self, min: f64, max: f64) -> f64 {
        self.rng.gen_range(min..max)
    }

    /// Returns a random element from the given slice.
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            return None;
        }
        let index = self.int(slice.len() as u32) as usize;
        Some(&slice[index])
    }

    /// Returns a random element from the given slice, or panics if the slice is empty.
    pub fn choose_unwrap<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        self.choose(slice).expect("Cannot choose from empty slice")
    }

    /// Shuffles the given slice in place.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        slice.shuffle(&mut self.rng);
    }

    /// Returns a random value using the given distribution.
    pub fn sample<T, D: Distribution<T>>(&mut self, distribution: D) -> T {
        distribution.sample(&mut self.rng)
    }

    /// Returns a random value that implements the Standard distribution.
    pub fn gen<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        self.rng.gen()
    }

    /// Returns a random value in the given range.
    pub fn gen_range<T>(&mut self, min: T, max: T) -> T
    where
        T: rand::distributions::uniform::SampleUniform + PartialOrd,
    {
        self.rng.gen_range(min..max)
    }

    /// Returns a random value using the uniform distribution.
    pub fn uniform<T>(&mut self, range: Range<T>) -> T
    where
        T: rand::distributions::uniform::SampleUniform,
    {
        let dist = Uniform::from(range);
        self.sample(dist)
    }

    /// Returns a weighted random index based on the given weights.
    ///
    /// The weights are normalized internally, so they don't need to sum to 1.0.
    pub fn weighted_index(&mut self, weights: &[f64]) -> Option<usize> {
        WeightedIndex::new(weights)
            .ok()
            .map(|distribution| distribution.sample(&mut self.rng))
    }

    /// Returns a weighted random choice from the given items and weights.
    pub fn weighted_choice<'a, T>(&mut self, items: &'a [T], weights: &[f64]) -> Option<&'a T> {
        if items.len() != weights.len() {
            return None;
        }
        self.weighted_index(weights).map(move |i| &items[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_results() {
        let seed = 42;
        let mut rng1 = SimRng::new(seed);
        let mut rng2 = SimRng::new(seed);

        // Both RNGs should produce the same sequence
        for _ in 0..100 {
            assert_eq!(rng1.int(100), rng2.int(100));
            assert_eq!(rng1.float(), rng2.float());
        }
    }

    #[test]
    fn test_different_seeds_different_results() {
        let mut rng1 = SimRng::new(42);
        let mut rng2 = SimRng::new(43);

        // Different seeds should produce different sequences
        let mut all_same = true;
        for _ in 0..10 {
            if rng1.int(1000) != rng2.int(1000) {
                all_same = false;
                break;
            }
        }
        assert!(
            !all_same,
            "Different seeds should produce different sequences"
        );
    }

    #[test]
    fn weighted_choice_rejects_mismatched_lengths() {
        let mut rng = SimRng::new(42);

        assert_eq!(rng.weighted_choice(&["only item"], &[1.0, 1.0]), None);
    }

    #[test]
    fn weighted_index_rejects_invalid_weights() {
        let mut rng = SimRng::new(42);

        assert_eq!(rng.weighted_index(&[]), None);
        assert_eq!(rng.weighted_index(&[0.0, 0.0]), None);
        assert_eq!(rng.weighted_index(&[1.0, -1.0]), None);
        assert_eq!(rng.weighted_index(&[1.0, f64::NAN]), None);
    }
}
