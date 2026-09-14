/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::{Distribution, SampleRange, Uniform};
use deep_causality_rand::{Rng, RngCore, RngError};

// Mock Rng for deterministic testing
struct MockFloatRng {
    val: f64,
}

impl RngCore for MockFloatRng {
    fn next_u32(&mut self) -> u32 {
        (self.val * u32::MAX as f64) as u32
    }
    fn next_u64(&mut self) -> u64 {
        (self.val * u64::MAX as f64) as u64
    }
    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        unimplemented!()
    }
}

impl Rng for MockFloatRng {}

#[test]
fn test_f32_sample_single_in_range() {
    let mut rng = MockFloatRng { val: 0.5 };
    let range = 10.0f32..20.0f32;
    let sample = range.sample_single(&mut rng).unwrap();
    assert!((10.0..20.0).contains(&sample));

    // `random_range(a..b)` and `Uniform::new(a, b).sample()` are the same mathematical draw, so
    // they must return the same value. They did not before this change: the range path went
    // through a 24-bit multiply kernel and the `Uniform` path through a 23-bit transmute kernel,
    // and at this mock's word they returned 15.0 and 14.999999 respectively. Both are valid
    // uniform draws; having two of them behind two names for one operation was the defect.
    // Asserting the agreement rather than a literal keeps the two paths pinned together, and
    // cannot silently drift the way a magic constant can.
    let mut rng2 = MockFloatRng { val: 0.5 };
    let via_uniform: f32 = Uniform::new(10.0f32, 20.0f32).unwrap().sample(&mut rng2);
    assert_eq!(sample, via_uniform, "the two range APIs must agree");
}

#[test]
fn test_f32_sample_single_empty_range_error() {
    let mut rng = MockFloatRng { val: 0.5 };
    let range = 20.0f32..10.0f32;
    let res = range.sample_single(&mut rng);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        RngError::InvalidRange("Invalid range: low must be less than high".to_string())
    );
}

#[test]
fn test_f32_is_empty() {
    let range_empty = 10.0f32..10.0f32;
    assert!(range_empty.is_empty());

    let range_valid = 10.0f32..20.0f32;
    assert!(!range_valid.is_empty());

    let range_inverted = 20.0f32..10.0f32;
    assert!(range_inverted.is_empty());
}

#[test]
fn test_f64_sample_single_in_range() {
    let mut rng = MockFloatRng { val: 0.5 };
    let range = 10.0f64..20.0f64;
    let sample = range.sample_single(&mut rng).unwrap();
    assert!((10.0..20.0).contains(&sample));

    // The same agreement as the `f32` case above.
    let mut rng2 = MockFloatRng { val: 0.5 };
    let via_uniform: f64 = Uniform::new(10.0f64, 20.0f64).unwrap().sample(&mut rng2);
    assert_eq!(sample, via_uniform, "the two range APIs must agree");
}

#[test]
fn test_f64_sample_single_empty_range_error() {
    let mut rng = MockFloatRng { val: 0.5 };
    let range = 20.0f64..10.0f64;
    let res = range.sample_single(&mut rng);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        RngError::InvalidRange("Invalid range: low must be less than high".to_string())
    );
}

#[test]
fn test_f64_is_empty() {
    let range_empty = 10.0f64..10.0f64;
    assert!(range_empty.is_empty());

    let range_valid = 10.0f64..20.0f64;
    assert!(!range_valid.is_empty());

    let range_inverted = 20.0f64..10.0f64;
    assert!(range_inverted.is_empty());
}
