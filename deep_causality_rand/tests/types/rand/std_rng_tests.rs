/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::types::Xoshiro256;
use deep_causality_rand::{Rng, RngCore};

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
fn test_xoshiro256_gen_range_produces_value_in_range() {
    let mut rng = Xoshiro256::new();
    let range = 10u64..20u64; // Explicitly use u64
    let val = rng.random_range(range.clone()); // Changed gen_range to random_range
    assert!(
        val >= range.start && val < range.end,
        "gen_range should produce a value within the specified range"
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
fn test_xoshiro256_gen_range_invalid_range_panics() {
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
