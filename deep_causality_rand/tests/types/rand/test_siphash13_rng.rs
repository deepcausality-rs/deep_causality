/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::{Rng, RngCore};
// Import Rng and RngCore
use deep_causality_rand::types::SipHash13Rng;

#[test]
fn test_siphash13_rng_next_u64_produces_non_zero() {
    let mut rng = SipHash13Rng::new();
    let val = rng.next_u64();
    // This test should fail because the current implementation always returns 0
    assert_ne!(val, 0, "next_u64 should produce a non-zero value");
}

#[test]
fn test_siphash13_rng_gen_range_produces_value_in_range() {
    let mut rng = SipHash13Rng::new();
    let range = 10u64..20u64; // Explicitly use u64
    let val = rng.random_range(range.clone()); // Changed gen_range to random_range
    // This test should fail because the current implementation always returns 0, which is not in 10..20
    assert!(
        val >= range.start && val < range.end,
        "gen_range should produce a value within the specified range"
    );
}

#[test]
fn test_siphash13_rng_next_u64_produces_different_values() {
    let mut rng = SipHash13Rng::new();
    let val1 = rng.next_u64();
    let val2 = rng.next_u64();
    // This test should fail because the current implementation always returns 0
    assert_ne!(
        val1, val2,
        "Consecutive calls to next_u64 should produce different values"
    );
}

#[test]
#[should_panic(expected = "cannot sample empty range")] // Changed expected panic message
fn test_siphash13_rng_gen_range_invalid_range_panics() {
    let mut rng = SipHash13Rng::new();
    let _ = rng.random_range(10u64..10u64); // Changed gen_range to random_range
}
