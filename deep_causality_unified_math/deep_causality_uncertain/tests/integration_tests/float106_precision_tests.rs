/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! End-to-end precision tests for the `Float106` instantiation of the uncertain engine:
//! the two load-bearing paths (certain values and deterministic arithmetic) carry full
//! double-double precision through the closed `SampledValue` dispatcher, and the
//! `MaybeUncertain<Float106>` present/dropout surface works.

use deep_causality_num::Float106;
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

/// `1/3` at double-double precision: its low limb is nonzero, so it is unrepresentable in
/// f64 and exercises the precision-carrying path.
fn one_third() -> Float106 {
    Float106::from_f64(1.0) / Float106::from_f64(3.0)
}

#[test]
fn certain_float106_is_lossless() {
    let x = one_third();
    assert_ne!(x.lo(), 0.0, "test value must exercise the low limb");

    let u = Uncertain::<Float106>::point(x);
    let s = u.sample().unwrap();

    assert_eq!(s, x, "certain Float106 sampled losslessly");
    assert_ne!(s.lo(), 0.0, "low limb preserved through the cache + sampler");
}

#[test]
fn float106_arithmetic_preserves_precision() {
    let third = one_third();
    let a = Uncertain::<Float106>::point(third);
    let b = Uncertain::<Float106>::point(third);

    let sum = (a + b).sample().unwrap();
    let expected = third + third;

    assert_eq!(sum, expected, "Float106 arithmetic composed at full precision");
    assert_ne!(sum.lo(), 0.0, "the double-double tail survived the arithmetic node");
}

#[test]
fn float106_normal_samples_are_finite_and_double_double() {
    let u = Uncertain::<Float106>::normal(Float106::from_f64(0.0), Float106::from_f64(1.0));
    let samples = u.take_samples(500).unwrap();

    assert!(samples.iter().all(|s| s.is_finite()), "all normal draws finite");
    assert!(
        samples.iter().any(|s| s.lo() != 0.0),
        "normal draws carry double-double entropy (low limb populated)"
    );
}

#[test]
fn float106_uniform_samples_in_range() {
    let low = Float106::from_f64(10.0);
    let high = Float106::from_f64(20.0);
    let u = Uncertain::<Float106>::uniform(low, high);
    for s in u.take_samples(500).unwrap() {
        assert!(s >= low && s < high, "uniform sample {s:?} out of [10, 20)");
    }
}

#[test]
fn maybe_uncertain_float106_present_value_and_lift() {
    let x = Float106::from_f64(2.0) / Float106::from_f64(7.0);
    let m = MaybeUncertain::<Float106>::from_value(x);

    assert_eq!(m.sample().unwrap(), Some(x), "present value sampled losslessly");

    // A certainly-present value lifts to a plain Uncertain<Float106> at full precision.
    let lifted = m.lift_to_uncertain(0.5, 0.95, 0.05, 1000).unwrap();
    assert_eq!(lifted.sample().unwrap(), x);
}

#[test]
fn maybe_uncertain_float106_dropout_does_not_lift() {
    let none = MaybeUncertain::<Float106>::always_none();
    assert_eq!(none.sample().unwrap(), None, "absent value samples to None");
    assert!(
        none.lift_to_uncertain(0.5, 0.95, 0.05, 1000).is_err(),
        "an absent value fails the presence gate (the dropout signal)"
    );
}
