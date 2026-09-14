/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The log-normal distribution, checked against its closed form.

use deep_causality_num::Float106;
use deep_causality_stats::LogNormal;
use deep_causality_stats::utils_tests::sampling::{
    SUITE_SEED, assert_near, draws, expect_accepted, expect_refused, lift, moments, quantile,
    sorted_draws,
};

const N: u64 = 200_000;

// ---------------------------------------------------------------------------------------------
// The closed form
// ---------------------------------------------------------------------------------------------

#[test]
fn the_mean_is_exp_mu_plus_half_sigma_squared() {
    // `σ = 0.5`, not 0: at `σ = 0` the distribution is a point mass and the mean and median
    // coincide, so a sampler confusing them would pass. `mean = e^{0 + 0.125} = 1.1331`.
    let d = expect_accepted(LogNormal::new(0.0f64, 0.5f64), "mu 0, sigma 0.5");
    let (mean, _) = moments::<f64, _>(&d, N, SUITE_SEED);
    assert_near(mean, (0.125f64).exp(), 0.02, "LogNormal(0, 0.5) mean");
}

#[test]
fn the_median_is_exp_mu_and_differs_from_the_mean() {
    // The usual error with this distribution is to assert `e^μ` for the mean. Both are pinned,
    // and at `σ = 0.5` they differ by 13%, which is far outside either tolerance.
    let d = expect_accepted(LogNormal::new(0.0f64, 0.5f64), "mu 0, sigma 0.5");
    let xs = sorted_draws::<f64, _>(&d, N, SUITE_SEED);
    let median = quantile(&xs, 0.5);
    assert_near(median, 1.0, 0.02, "LogNormal(0, 0.5) median = e^0");

    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    assert!(
        mean > median * 1.05,
        "the mean {mean} should exceed the median {median} by about e^{{sigma^2/2}} = 1.133"
    );
}

#[test]
fn a_non_zero_mu_shifts_the_median_multiplicatively() {
    // `median = e^μ`. At `μ = 1` that is 2.718, so a sampler that adds `μ` after exponentiating
    // rather than before fails here.
    let d = expect_accepted(LogNormal::new(1.0f64, 0.25f64), "mu 1, sigma 0.25");
    let xs = sorted_draws::<f64, _>(&d, N, SUITE_SEED);
    assert_near(
        quantile(&xs, 0.5),
        std::f64::consts::E,
        0.02,
        "median = e^1",
    );
}

// ---------------------------------------------------------------------------------------------
// The support
// ---------------------------------------------------------------------------------------------

#[test]
fn every_draw_is_strictly_positive() {
    // The support is `(0, ∞)`. A non-positive value is a structural failure rather than a
    // statistical one — it means the exponential was not applied — so this is asserted on **every**
    // draw rather than on a moment, which would average the failure away.
    let d = expect_accepted(LogNormal::new(0.0f64, 1.0f64), "mu 0, sigma 1");
    for (i, x) in draws::<f64, _>(&d, 20_000, SUITE_SEED)
        .into_iter()
        .enumerate()
    {
        assert!(x > 0.0, "draw {i} was {x}, outside the support (0, inf)");
        assert!(x.is_finite(), "draw {i} was {x}");
    }
}

#[test]
fn the_logarithm_of_a_draw_is_normal() {
    // The defining property: `ln(X) ~ N(μ, σ²)`. A sampler with the right support and the wrong
    // shape fails here where a positivity check would not.
    let (mu, sigma) = (0.5f64, 0.75f64);
    let d = expect_accepted(LogNormal::new(mu, sigma), "mu 0.5, sigma 0.75");
    let logs: Vec<f64> = draws::<f64, _>(&d, N, SUITE_SEED)
        .into_iter()
        .map(f64::ln)
        .collect();

    let log_mean = logs.iter().sum::<f64>() / logs.len() as f64;
    let log_var = logs
        .iter()
        .map(|x| (x - log_mean) * (x - log_mean))
        .sum::<f64>()
        / logs.len() as f64;

    assert_near(log_mean, mu, 0.02, "mean of ln(X)");
    assert_near(log_var, sigma * sigma, 0.05, "variance of ln(X)");
}

// ---------------------------------------------------------------------------------------------
// Every scalar
// ---------------------------------------------------------------------------------------------

#[test]
fn the_same_body_samples_every_scalar() {
    let want = (0.125f64).exp();

    let d32 = expect_accepted(LogNormal::new(0.0f32, 0.5f32), "f32");
    let (m32, _) = moments::<f32, _>(&d32, 50_000, SUITE_SEED);
    assert_near(m32, want, 0.03, "f32 mean");

    let d64 = expect_accepted(LogNormal::new(0.0f64, 0.5f64), "f64");
    let (m64, _) = moments::<f64, _>(&d64, 50_000, SUITE_SEED);
    assert_near(m64, want, 0.03, "f64 mean");

    let d106 = expect_accepted(
        LogNormal::new(lift::<Float106>(0.0), lift::<Float106>(0.5)),
        "Float106",
    );
    let (m106, _) = moments::<Float106, _>(&d106, 50_000, SUITE_SEED);
    assert_near(m106, want, 0.03, "Float106 mean");
}

// ---------------------------------------------------------------------------------------------
// Parameters outside the support
// ---------------------------------------------------------------------------------------------

#[test]
fn a_non_positive_sigma_is_refused() {
    expect_refused(LogNormal::new(0.0f64, 0.0f64), "a zero sigma");
    expect_refused(LogNormal::new(0.0f64, -1.0f64), "a negative sigma");
}

#[test]
fn a_non_finite_parameter_is_refused() {
    expect_refused(LogNormal::new(f64::NAN, 1.0f64), "a NaN mu");
    expect_refused(LogNormal::new(0.0f64, f64::NAN), "a NaN sigma");
    expect_refused(LogNormal::new(f64::INFINITY, 1.0f64), "an infinite mu");
    expect_refused(LogNormal::new(0.0f64, f64::INFINITY), "an infinite sigma");
}

#[test]
fn valid_parameters_are_kept_verbatim() {
    let d = expect_accepted(LogNormal::new(1.5f64, 2.5f64), "mu 1.5, sigma 2.5");
    assert_eq!(d.mu(), 1.5, "the constructor altered mu");
    assert_eq!(d.sigma(), 2.5, "the constructor altered sigma");
}
