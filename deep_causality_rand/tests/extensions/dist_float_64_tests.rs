/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::types::Xoshiro256;
use deep_causality_rand::{Distribution, Open01, OpenClosed01, StandardUniform};

#[test]
fn test_standard_uniform_f64_sample_range() {
    let mut rng = Xoshiro256::new();
    let dist = StandardUniform;

    for _ in 0..1000 {
        let val: f64 = dist.sample(&mut rng);
        assert!(
            (0.0..1.0).contains(&val),
            "Value {} out of range [0.0, 1.0)",
            val
        );
    }
}

#[test]
fn test_standard_uniform_f64_sample_randomness() {
    let mut rng = Xoshiro256::new();
    let dist = StandardUniform;

    let mut values = Vec::new();
    for _ in 0..1000 {
        values.push(dist.sample(&mut rng));
    }

    // Check for some basic randomness (e.g., not all same, not sequential)
    assert!(
        values.iter().any(|&x: &f64| x != values[0]),
        "Not all values are the same"
    );
    assert!(
        values.windows(2).any(|w| w[0] != w[1]),
        "Values are not sequential"
    );
}

#[test]
fn test_open_closed_01_f64_sample_range() {
    let mut rng = Xoshiro256::new();
    let dist = OpenClosed01;

    for _ in 0..1000 {
        let val: f64 = dist.sample(&mut rng);
        assert!(
            val > 0.0 && val <= 1.0,
            "Value {} out of range (0.0, 1.0]",
            val
        );
    }
}

#[test]
fn test_open_closed_01_f64_sample_randomness() {
    let mut rng = Xoshiro256::new();
    let dist = OpenClosed01;

    let mut values = Vec::new();
    for _ in 0..1000 {
        values.push(dist.sample(&mut rng));
    }

    assert!(
        values.iter().any(|&x: &f64| x != values[0]),
        "Not all values are the same"
    );
    assert!(
        values.windows(2).any(|w| w[0] != w[1]),
        "Values are not sequential"
    );
}

#[test]
fn test_open01_f64_sample_range() {
    let mut rng = Xoshiro256::new();
    let dist = Open01;

    for _ in 0..1000 {
        let val: f64 = dist.sample(&mut rng);
        assert!(
            val > 0.0 && val < 1.0,
            "Value {} out of range (0.0, 1.0)",
            val
        );
    }
}

#[test]
fn test_open01_f64_sample_randomness() {
    let mut rng = Xoshiro256::new();
    let dist = Open01;

    let mut values = Vec::new();
    for _ in 0..1000 {
        values.push(dist.sample(&mut rng));
    }

    assert!(
        values.iter().any(|&x: &f64| x != values[0]),
        "Not all values are the same"
    );
    assert!(
        values.windows(2).any(|w| w[0] != w[1]),
        "Values are not sequential"
    );
}
