/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The exponential distribution, checked against its closed form.

use deep_causality_num::Float106;
use deep_causality_rand::Distribution;
use deep_causality_stats::utils_tests::sampling::{
    SUITE_SEED, ZeroRng, assert_guards_against_zero, assert_near, draws, expect_accepted,
    expect_refused, lift, moments,
};
use deep_causality_stats::{Exponential, StandardUniform};

const N: u64 = 200_000;

// ---------------------------------------------------------------------------------------------
// The closed form
// ---------------------------------------------------------------------------------------------

#[test]
fn the_mean_is_the_reciprocal_rate() {
    // Rate 2, not 1: at rate 1 the rate and the scale coincide, so a sampler that confuses them
    // would pass. `mean = 1/λ = 0.5`.
    let d = expect_accepted(Exponential::new(2.0f64), "rate 2");
    let (mean, _) = moments::<f64, _>(&d, N, SUITE_SEED);
    assert_near(mean, 0.5, 0.01, "Exponential(2) mean");
}

#[test]
fn the_variance_is_the_squared_reciprocal_rate() {
    // `variance = 1/λ² = 0.25`. A sampler can get the mean right and the spread wrong.
    let d = expect_accepted(Exponential::new(2.0f64), "rate 2");
    let (_, var) = moments::<f64, _>(&d, N, SUITE_SEED);
    assert_near(var, 0.25, 0.05, "Exponential(2) variance");
}

#[test]
fn the_rate_is_the_rate_and_not_the_scale() {
    // The two readings of the parameter differ everywhere except λ = 1. At λ = 4 the rate reading
    // gives a mean of 0.25 and the scale reading a mean of 4.
    let d = expect_accepted(Exponential::new(4.0f64), "rate 4");
    let (mean, _) = moments::<f64, _>(&d, N, SUITE_SEED);
    assert_near(
        mean,
        0.25,
        0.01,
        "Exponential(4) mean under the rate reading",
    );
}

// ---------------------------------------------------------------------------------------------
// Every scalar
// ---------------------------------------------------------------------------------------------

#[test]
fn the_same_body_samples_every_scalar() {
    let f32_d = expect_accepted(Exponential::new(2.0f32), "f32 rate");
    let (m32, _) = moments::<f32, _>(&f32_d, 50_000, SUITE_SEED);
    assert_near(m32, 0.5, 0.02, "f32 mean");

    let f64_d = expect_accepted(Exponential::new(2.0f64), "f64 rate");
    let (m64, _) = moments::<f64, _>(&f64_d, 50_000, SUITE_SEED);
    assert_near(m64, 0.5, 0.02, "f64 mean");

    let f106_d = expect_accepted(Exponential::new(lift::<Float106>(2.0)), "Float106 rate");
    let (m106, _) = moments::<Float106, _>(&f106_d, 50_000, SUITE_SEED);
    assert_near(m106, 0.5, 0.02, "Float106 mean");
}

// ---------------------------------------------------------------------------------------------
// Support and the logarithm
// ---------------------------------------------------------------------------------------------

#[test]
fn every_draw_is_finite_and_non_negative() {
    // The support is `[0, ∞)`. An infinity here would come from `ln(0)`, which is the reason the
    // draw is taken from the open interval rather than the half-open one — and which no assertion
    // on a *mean* would catch, because one infinity poisons the mean into `NaN` and a `NaN`
    // comparison is false rather than loud.
    let d = expect_accepted(Exponential::new(1.0f64), "rate 1");
    for (i, x) in draws::<f64, _>(&d, 10_000, SUITE_SEED)
        .into_iter()
        .enumerate()
    {
        assert!(x.is_finite(), "draw {i} was {x}");
        assert!(x >= 0.0, "draw {i} was negative: {x}");
    }
}

#[test]
fn the_distribution_is_memoryless() {
    // `P(X > s + t | X > s) = P(X > t)`, the property that characterises the exponential among
    // all continuous distributions. A sampler with the right mean and the wrong shape fails here.
    let d = expect_accepted(Exponential::new(1.0f64), "rate 1");
    let xs = draws::<f64, _>(&d, N, SUITE_SEED);

    let (s, t) = (0.5f64, 1.0f64);
    let beyond_s: Vec<f64> = xs.iter().copied().filter(|x| *x > s).collect();
    let conditional =
        beyond_s.iter().filter(|x| **x > s + t).count() as f64 / beyond_s.len() as f64;
    let unconditional = xs.iter().filter(|x| **x > t).count() as f64 / xs.len() as f64;

    assert_near(conditional, unconditional, 0.02, "memorylessness");
}

// ---------------------------------------------------------------------------------------------
// Parameters outside the support
// ---------------------------------------------------------------------------------------------

#[test]
fn a_non_positive_rate_is_refused() {
    expect_refused(Exponential::new(0.0f64), "a zero rate");
    expect_refused(Exponential::new(-1.0f64), "a negative rate");
}

#[test]
fn a_non_finite_rate_is_refused() {
    expect_refused(Exponential::new(f64::NAN), "a NaN rate");
    expect_refused(Exponential::new(f64::INFINITY), "an infinite rate");
    expect_refused(
        Exponential::new(f64::NEG_INFINITY),
        "a negative infinite rate",
    );
}

#[test]
fn a_valid_rate_is_kept_verbatim() {
    let d = expect_accepted(Exponential::new(2.5f64), "rate 2.5");
    assert_eq!(d.rate(), 2.5, "the constructor altered its parameter");
}

// ---------------------------------------------------------------------------------------------
// The harness itself
// ---------------------------------------------------------------------------------------------

#[test]
fn the_harness_is_deterministic() {
    // Every suite in this group depends on it: a failure that cannot be reproduced is a failure
    // nobody can act on.
    let a = draws::<f64, _>(&StandardUniform, 100, SUITE_SEED);
    let b = draws::<f64, _>(&StandardUniform, 100, SUITE_SEED);
    assert_eq!(a, b, "the same seed gave two different samples");
}

#[test]
fn the_draw_is_taken_from_the_open_interval() {
    // `-ln(u)/λ` with `u` from the half-open `[0, 1)` gives an infinity when `u` is zero. That
    // happens at probability `2^-53`, so the 10 000-draw finiteness test above cannot reach it —
    // it passes against the defective form. A rigged generator reaches it immediately.
    assert_guards_against_zero(
        || {
            let d = Exponential::new(1.0f64).expect("rate 1");
            let _: f64 = d.sample(&mut ZeroRng);
        },
        "Exponential",
    );
}
