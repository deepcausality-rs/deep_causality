/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::types::Xoshiro256;
use deep_causality_rand::{Distribution, Rng, RngCore, StandardUniform};

#[test]
fn test_xoshiro256_default() {
    let _rng = Xoshiro256::default();
    // This covers the default() implementation.
}

#[test]
fn test_xoshiro256_next_u32() {
    let mut rng = Xoshiro256::new();
    let val = rng.next_u32();
    // We cannot assert a specific value, but we can assert it's not zero (highly unlikely for a random u32)
    // and that it doesn't panic.
    assert_ne!(
        val, 0,
        "next_u32 should produce a non-zero value (highly probable)"
    );
}

#[test]
fn test_xoshiro256_next_u64_produces_non_zero() {
    let mut rng = Xoshiro256::new();
    let val = rng.next_u64();
    assert_ne!(val, 0, "next_u64 should produce a non-zero value");
}

#[test]
fn test_xoshiro256_random_range_produces_value_in_range() {
    let mut rng = Xoshiro256::new();
    let range = 10u64..20u64; // Explicitly use u64
    let val = rng.random_range(range.clone());
    assert!(
        val >= range.start && val < range.end,
        "random_range should produce a value within the specified range"
    );
}

#[test]
fn test_xoshiro256_random_range_single_value_range() {
    let mut rng = Xoshiro256::new();
    let range = 5u32..6u32;
    let val = rng.random_range(range.clone());
    assert_eq!(val, 5, "random_range for 5..6 should always return 5");
}

#[test]
fn test_xoshiro256_random_range_zero_to_one() {
    let mut rng = Xoshiro256::new();
    let range = 0.0f64..1.0f64;
    let val = rng.random_range(range.clone());
    assert!(
        val >= range.start && val < range.end,
        "random_range for 0.0..1.0 should produce a value within the specified range"
    );
}

#[test]
fn test_xoshiro256_next_u64_produces_different_values() {
    let mut rng = Xoshiro256::new();
    let val1 = rng.next_u64();
    let val2 = rng.next_u64();
    assert_ne!(
        val1, val2,
        "Consecutive calls to next_u64 should produce different values (highly probable)"
    );
}

#[test]
#[should_panic(expected = "cannot sample empty range")]
fn test_xoshiro256_random_range_empty_panics() {
    let mut rng = Xoshiro256::new();
    let _ = rng.random_range(10u64..10u64);
}

#[test]
fn test_xoshiro256_fill_bytes() {
    let mut rng = Xoshiro256::new();
    let mut buffer = [0u8; 16];
    rng.fill_bytes(&mut buffer);
    // Cannot assert specific values, but can assert it's not all zeros (highly probable)
    assert!(
        !buffer.iter().all(|&x| x == 0),
        "Buffer should not be all zeros"
    );
}

#[test]
fn test_xoshiro256_fill_bytes_different_sizes() {
    let mut rng = Xoshiro256::new();
    let mut buffer_small = [0u8; 4];
    rng.fill_bytes(&mut buffer_small);
    assert!(
        !buffer_small.iter().all(|&x| x == 0),
        "Small buffer should not be all zeros"
    );

    let mut buffer_large = [0u8; 100];
    rng.fill_bytes(&mut buffer_large);
    assert!(
        !buffer_large.iter().all(|&x| x == 0),
        "Large buffer should not be all zeros"
    );
}

#[test]
fn test_xoshiro256_random_bool() {
    let mut rng = Xoshiro256::new();
    let mut true_count = 0;
    let mut false_count = 0;
    const NUM_SAMPLES: usize = 1000;

    for _ in 0..NUM_SAMPLES {
        if rng.random_bool(0.5) {
            true_count += 1;
        } else {
            false_count += 1;
        }
    }
    // Check for rough balance
    assert!((true_count as f64 - false_count as f64).abs() < (NUM_SAMPLES as f64 * 0.2));
}

#[test]
fn test_xoshiro256_random_bool_edge_cases() {
    let mut rng = Xoshiro256::new();
    assert!(rng.random_bool(1.0), "Should always be true for p=1.0");
    assert!(!rng.random_bool(0.0), "Should always be false for p=0.0");

    // Test panic for invalid p
    let mut rng_panic = Xoshiro256::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rng_panic.random_bool(1.1);
    }));
    assert!(result.is_err(), "Should panic for p > 1.0");

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rng_panic.random_bool(-0.1);
    }));
    assert!(result.is_err(), "Should panic for p < 0.0");
}

#[test]
fn test_xoshiro256_random_ratio() {
    let mut rng = Xoshiro256::new();
    let mut true_count = 0;
    let mut false_count = 0;
    const NUM_SAMPLES: usize = 1000;

    for _ in 0..NUM_SAMPLES {
        if rng.random_ratio(1, 2) {
            true_count += 1;
        } else {
            false_count += 1;
        }
    }
    // Check for rough balance (1/2 ratio)
    assert!((true_count as f64 - false_count as f64).abs() < (NUM_SAMPLES as f64 * 0.2));
}

#[test]
fn test_xoshiro256_random_ratio_edge_cases() {
    let mut rng = Xoshiro256::new();
    assert!(
        rng.random_ratio(1, 1),
        "Should always be true for 1/1 ratio"
    );
    assert!(
        !rng.random_ratio(0, 1),
        "Should always be false for 0/1 ratio"
    );

    // Test panic for invalid ratio
    let mut rng_panic = Xoshiro256::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rng_panic.random_ratio(2, 1);
    }));
    assert!(result.is_err(), "Should panic for numerator > denominator");

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rng_panic.random_ratio(1, 0);
    }));
    assert!(result.is_err(), "Should panic for denominator == 0");
}

#[test]
fn test_xoshiro256_random_u32() {
    let mut rng = Xoshiro256::new();
    let val: u32 = rng.random();
    assert_ne!(val, 0, "random() for u32 should produce a non-zero value");
}

#[test]
fn test_xoshiro256_random_f64() {
    let mut rng = Xoshiro256::new();
    let val: f64 = rng.random();
    assert!(
        (0.0..1.0).contains(&val),
        "random() for f64 should produce a value in [0.0, 1.0)"
    );
}

#[test]
fn test_xoshiro256_sample_iter_u32() {
    let mut rng = Xoshiro256::new();
    let mut iter = rng.sample_iter(StandardUniform);
    let val1: u32 = iter.next().unwrap();
    let val2: u32 = iter.next().unwrap();
    assert_ne!(val1, val2, "sample_iter should produce different values");
}

#[test]
fn test_xoshiro256_sample_iter_f64() {
    let mut rng = Xoshiro256::new();
    let mut iter = rng.sample_iter(StandardUniform);
    let val1: f64 = iter.next().unwrap();
    let val2: f64 = iter.next().unwrap();
    assert_ne!(val1, val2, "sample_iter should produce different values");
    assert!(
        (0.0..1.0).contains(&val1),
        "sample_iter for f64 should produce a value in [0.0, 1.0)"
    );
}

#[test]
fn test_xoshiro256_map_u32_to_u64() {
    let mut rng = Xoshiro256::new();
    let mapped_dist = rng.map(|x: u32| x as u64);
    let val: u64 = mapped_dist.sample(&mut rng);
    assert_ne!(val, 0, "Mapped u32 to u64 should produce a non-zero value");
}

#[test]
fn test_xoshiro256_map_f64_to_f32() {
    let mut rng = Xoshiro256::new();
    let mapped_dist = rng.map(|x: f64| x as f32);
    let val: f32 = mapped_dist.sample(&mut rng);
    assert!(
        (0.0..1.0).contains(&val),
        "Mapped f64 to f32 should produce a value in [0.0, 1.0)"
    );
}

#[test]
fn test_rng_random_iter() {
    let mut rng = Xoshiro256::new();
    let mut iter = rng.random_iter::<u32>();
    let val1 = iter.next().unwrap();
    let val2 = iter.next().unwrap();
    assert_ne!(val1, val2, "random_iter should produce different values");
}

#[test]
fn test_rng_fill() {
    let mut rng = Xoshiro256::new();
    let mut buffer = [0u8; 32];
    rng.fill_bytes(&mut buffer[..]);
    assert!(
        !buffer.iter().all(|&x| x == 0),
        "Buffer should not be all zeros after fill"
    );
}
