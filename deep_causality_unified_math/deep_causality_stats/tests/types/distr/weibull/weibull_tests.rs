/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Weibull distribution, checked against its closed form.

use deep_causality_num::Float106;
use deep_causality_stats::utils_tests::sampling::{
    SUITE_SEED, ZeroRng, assert_guards_against_zero, assert_near, draws, expect_accepted,
    expect_refused, lift, moments, quantile, sorted_draws,
};
use deep_causality_rand::Distribution;
use deep_causality_stats::{Exponential, Weibull};

const N: u64 = 200_000;

// ---------------------------------------------------------------------------------------------
// The closed form
// ---------------------------------------------------------------------------------------------

#[test]
fn the_mean_at_shape_two_is_lambda_root_pi_over_two() {
    // `mean = λ·Γ(1 + 1/k)`; at `k = 2` that is `λ·Γ(1.5) = λ·√π/2 = 0.8862`.
    let d = expect_accepted(Weibull::new(2.0f64, 1.0f64), "k 2, lambda 1");
    let (mean, _) = moments::<f64, _>(&d, N, SUITE_SEED);
    assert_near(mean, std::f64::consts::PI.sqrt() / 2.0, 0.01, "Weibull(2, 1) mean");
}

#[test]
fn the_scale_multiplies_the_mean() {
    // Doubling `λ` doubles the mean, which a sampler applying the scale inside the power does not.
    let d = expect_accepted(Weibull::new(2.0f64, 3.0f64), "k 2, lambda 3");
    let (mean, _) = moments::<f64, _>(&d, N, SUITE_SEED);
    assert_near(
        mean,
        3.0 * std::f64::consts::PI.sqrt() / 2.0,
        0.01,
        "Weibull(2, 3) mean",
    );
}

#[test]
fn the_median_is_lambda_times_ln_two_to_the_reciprocal_shape() {
    // `median = λ(ln 2)^{1/k}`. At `k = 2, λ = 1` that is `√(ln 2) = 0.8326`.
    let d = expect_accepted(Weibull::new(2.0f64, 1.0f64), "k 2, lambda 1");
    let xs = sorted_draws::<f64, _>(&d, N, SUITE_SEED);
    assert_near(
        quantile(&xs, 0.5),
        2.0f64.ln().sqrt(),
        0.02,
        "Weibull(2, 1) median",
    );
}

// ---------------------------------------------------------------------------------------------
// The analytic identity at k = 1
// ---------------------------------------------------------------------------------------------

#[test]
fn at_shape_one_it_is_the_exponential() {
    // `Weibull(1, λ) == Exponential(1/λ)`. This is the one place in this group where a second
    // implementation is a legitimate oracle: the identity is analytic, so agreement is evidence
    // rather than a sampler agreeing with itself.
    //
    // It also separates the two readings of the exponent. `(-ln u)^{1/k}` and `(-ln u)^k` agree at
    // `k = 1` and nowhere else, which is why the mean tests above use `k = 2`.
    let lambda = 2.0f64;
    let w = expect_accepted(Weibull::new(1.0f64, lambda), "k 1");
    let e = expect_accepted(Exponential::new(1.0 / lambda), "rate 1/lambda");

    let (wm, wv) = moments::<f64, _>(&w, N, SUITE_SEED);
    let (em, ev) = moments::<f64, _>(&e, N, SUITE_SEED);

    assert_near(wm, em, 0.02, "Weibull(1, 2) mean against Exponential(0.5)");
    assert_near(wv, ev, 0.05, "Weibull(1, 2) variance against Exponential(0.5)");
}

// ---------------------------------------------------------------------------------------------
// Support and the logarithm
// ---------------------------------------------------------------------------------------------

#[test]
fn every_draw_is_finite_and_non_negative() {
    let d = expect_accepted(Weibull::new(1.5f64, 1.0f64), "k 1.5");
    for (i, x) in draws::<f64, _>(&d, 10_000, SUITE_SEED).into_iter().enumerate() {
        assert!(x.is_finite(), "draw {i} was {x}");
        assert!(x >= 0.0, "draw {i} was negative: {x}");
    }
}

#[test]
fn the_draw_is_taken_from_the_open_interval() {
    // `(-ln u)^{1/k}` with `u` from the half-open draw gives an infinity at `u = 0`, which sits at
    // probability `2^-53` and so cannot be reached by sampling.
    assert_guards_against_zero(
        || {
            let d = Weibull::new(2.0f64, 1.0f64).expect("k 2");
            let _: f64 = d.sample(&mut ZeroRng);
        },
        "Weibull",
    );
}

// ---------------------------------------------------------------------------------------------
// Every scalar
// ---------------------------------------------------------------------------------------------

#[test]
fn the_same_body_samples_every_scalar() {
    let want = std::f64::consts::PI.sqrt() / 2.0;

    let d32 = expect_accepted(Weibull::new(2.0f32, 1.0f32), "f32");
    let (m32, _) = moments::<f32, _>(&d32, 50_000, SUITE_SEED);
    assert_near(m32, want, 0.02, "f32 mean");

    let d64 = expect_accepted(Weibull::new(2.0f64, 1.0f64), "f64");
    let (m64, _) = moments::<f64, _>(&d64, 50_000, SUITE_SEED);
    assert_near(m64, want, 0.02, "f64 mean");

    let d106 = expect_accepted(
        Weibull::new(lift::<Float106>(2.0), lift::<Float106>(1.0)),
        "Float106",
    );
    let (m106, _) = moments::<Float106, _>(&d106, 50_000, SUITE_SEED);
    assert_near(m106, want, 0.02, "Float106 mean");
}

// ---------------------------------------------------------------------------------------------
// Parameters outside the support
// ---------------------------------------------------------------------------------------------

#[test]
fn a_non_positive_shape_or_scale_is_refused() {
    expect_refused(Weibull::new(0.0f64, 1.0f64), "a zero shape");
    expect_refused(Weibull::new(-1.0f64, 1.0f64), "a negative shape");
    expect_refused(Weibull::new(1.0f64, 0.0f64), "a zero scale");
    expect_refused(Weibull::new(1.0f64, -1.0f64), "a negative scale");
}

#[test]
fn a_non_finite_parameter_is_refused() {
    expect_refused(Weibull::new(f64::NAN, 1.0f64), "a NaN shape");
    expect_refused(Weibull::new(1.0f64, f64::NAN), "a NaN scale");
    expect_refused(Weibull::new(f64::INFINITY, 1.0f64), "an infinite shape");
    expect_refused(Weibull::new(1.0f64, f64::INFINITY), "an infinite scale");
}

#[test]
fn valid_parameters_are_kept_verbatim() {
    let d = expect_accepted(Weibull::new(1.5f64, 2.5f64), "k 1.5, lambda 2.5");
    assert_eq!(d.shape(), 1.5);
    assert_eq!(d.scale(), 2.5);
}
