/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::{Distribution, Rng, RngCore, StandardUniform};

// Mock Rng for deterministic testing
struct MockRng {
    values: Vec<u32>,
    index: usize,
}

impl MockRng {
    fn new(values: Vec<u32>) -> Self {
        MockRng { values, index: 0 }
    }
}

impl RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        let val = self.values[self.index];
        self.index = (self.index + 1) % self.values.len();
        val
    }
    fn next_u64(&mut self) -> u64 {
        self.next_u32() as u64 // For simplicity, convert u32 to u64
    }
    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        unimplemented!()
    }
}

impl Rng for MockRng {}

// Mock Distribution for testing
#[derive(Clone, Copy)]
struct MockDistribution {
    value: u32,
}

impl Distribution<u32> for MockDistribution {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u32 {
        // In a real scenario, this would use rng to generate a sample.
        // For testing, we can make it predictable or use the rng directly.
        rng.next_u32() + self.value
    }
}

#[test]
fn test_distribution_sample() {
    let mut rng = MockRng::new(vec![10, 20, 30]);
    let dist = MockDistribution { value: 1 };
    let sample = dist.sample(&mut rng);
    assert_eq!(sample, 11);
    let sample = dist.sample(&mut rng);
    assert_eq!(sample, 21);
}

#[test]
fn test_distribution_sample_iter() {
    let mut rng = MockRng::new(vec![10, 20, 30]);
    let dist = MockDistribution { value: 1 };
    let mut iter = dist.sample_iter(&mut rng);

    assert_eq!(iter.next().unwrap(), 11);
    assert_eq!(iter.next().unwrap(), 21);
    assert_eq!(iter.next().unwrap(), 31);
    assert_eq!(iter.next().unwrap(), 11); // Should loop back due to MockRng
}

#[test]
fn test_distribution_map() {
    let mut rng = MockRng::new(vec![10, 20, 30]);
    let dist = MockDistribution { value: 1 };
    let mapped_dist = dist.map(|x| x * 2);

    let sample = mapped_dist.sample(&mut rng);
    assert_eq!(sample, 22); // (10 + 1) * 2

    let sample = mapped_dist.sample(&mut rng);
    assert_eq!(sample, 42); // (20 + 1) * 2
}

#[test]
fn test_distribution_map_with_standard_uniform() {
    let mut rng = MockRng::new(vec![10, 20, 30]);
    let dist = StandardUniform;
    let mapped_dist = dist.map(|x: u32| x as f32 / 100.0);

    let sample = mapped_dist.sample(&mut rng);
    assert_eq!(sample, 0.10);

    let sample = mapped_dist.sample(&mut rng);
    assert_eq!(sample, 0.20);
}
