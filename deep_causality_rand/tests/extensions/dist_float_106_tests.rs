/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Float106, One, Zero};
use deep_causality_rand::types::Xoshiro256;
use deep_causality_rand::{Distribution, Open01, OpenClosed01, StandardUniform};

#[test]
fn test_standard_uniform_f106_sample_range() {
    let mut rng = Xoshiro256::new();
    let dist = StandardUniform;
    let (zero, one) = (Float106::zero(), Float106::one());

    for _ in 0..1000 {
        let val: Float106 = dist.sample(&mut rng);
        assert!(
            val >= zero && val < one,
            "Value {val:?} out of range [0, 1)"
        );
    }
}

#[test]
fn test_open_closed_01_f106_sample_range() {
    let mut rng = Xoshiro256::new();
    let dist = OpenClosed01;
    let (zero, one) = (Float106::zero(), Float106::one());

    for _ in 0..1000 {
        let val: Float106 = dist.sample(&mut rng);
        assert!(
            val > zero && val <= one,
            "Value {val:?} out of range (0, 1]"
        );
    }
}

#[test]
fn test_open01_f106_sample_range() {
    let mut rng = Xoshiro256::new();
    let dist = Open01;
    let (zero, one) = (Float106::zero(), Float106::one());

    for _ in 0..1000 {
        let val: Float106 = dist.sample(&mut rng);
        assert!(val > zero && val < one, "Value {val:?} out of range (0, 1)");
    }
}

#[test]
fn test_f106_sample_randomness() {
    let mut rng = Xoshiro256::new();
    let dist = StandardUniform;

    let values: Vec<Float106> = (0..1000).map(|_| dist.sample(&mut rng)).collect();
    assert!(
        values.iter().any(|&x| x != values[0]),
        "Not all values are the same"
    );
    assert!(
        values.windows(2).any(|w| w[0] != w[1]),
        "Values are not sequential"
    );
}
