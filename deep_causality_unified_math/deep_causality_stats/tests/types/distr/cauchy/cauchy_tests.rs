/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Cauchy distribution, checked **by quantile only**.
//!
//! # There is no moment assertion in this file, and there must not be
//!
//! The Cauchy distribution has no mean and no variance: both defining integrals diverge. A sample
//! mean does not converge as the sample grows — it wanders, and its own distribution is Cauchy
//! again with the same parameters.
//!
//! So an assertion on a sample mean is not a weak test but a **meaningless** one. It passes or
//! fails according to the seed. It survives review, because it looks like every other moment test
//! in this crate, and then fails intermittently in CI where nobody can reproduce it.
//!
//! `seed_instability_demonstrates_why_there_is_no_mean_test` below measures that directly: the
//! sample mean across five seeds, printed, with the spread asserted to be large. That test is the
//! evidence for this prohibition, and it is why the prohibition is recorded here rather than
//! merely asserted.
//!
//! The parameters are recoverable without moments: the location is the median, and the scale is
//! half the interquartile range.
//!
//! This prohibition cannot be enforced mechanically without scraping the source, which this
//! repository forbids for good reason. It is enforced by being written where it will be read: if a
//! later change adds an assertion on a sample mean or variance to this module, this paragraph and
//! `seed_instability_demonstrates_why_there_is_no_mean_test` are what should stop it in review.

use deep_causality_num::Float106;
use deep_causality_stats::Cauchy;
use deep_causality_stats::utils_tests::sampling::{
    SUITE_SEED, assert_near, draws, expect_accepted, expect_refused, lift, quantile, sorted_draws,
};

const N: u64 = 20_000;

// ---------------------------------------------------------------------------------------------
// Quantiles
// ---------------------------------------------------------------------------------------------

#[test]
fn the_median_is_the_location() {
    let d = expect_accepted(Cauchy::new(0.0f64, 1.0f64), "standard Cauchy");
    let xs = sorted_draws::<f64, _>(&d, N, SUITE_SEED);
    let median = quantile(&xs, 0.5);
    assert!(median.abs() < 0.05, "median {median} should be near 0");
}

#[test]
fn the_interquartile_range_is_twice_the_scale() {
    let d = expect_accepted(Cauchy::new(0.0f64, 1.0f64), "standard Cauchy");
    let xs = sorted_draws::<f64, _>(&d, N, SUITE_SEED);
    let iqr = quantile(&xs, 0.75) - quantile(&xs, 0.25);
    assert_near(iqr, 2.0, 0.05, "interquartile range = 2 * scale");
}

#[test]
fn a_shifted_and_scaled_instance_is_recovered_from_its_quantiles() {
    // The parameters are identifiable, just not through moments.
    let (loc, scale) = (3.0f64, 2.0f64);
    let d = expect_accepted(Cauchy::new(loc, scale), "Cauchy(3, 2)");
    let xs = sorted_draws::<f64, _>(&d, N, SUITE_SEED);

    assert_near(quantile(&xs, 0.5), loc, 0.05, "median = location");
    let iqr = quantile(&xs, 0.75) - quantile(&xs, 0.25);
    assert_near(iqr, 2.0 * scale, 0.05, "IQR = 2 * scale");
}

#[test]
fn the_tails_are_heavy() {
    // `P(|X| > 1) = 1/2` for the standard Cauchy — half the mass sits outside the interquartile
    // range. A normal sampler mistaken for this one would put about 32% there.
    let d = expect_accepted(Cauchy::new(0.0f64, 1.0f64), "standard Cauchy");
    let xs = draws::<f64, _>(&d, 200_000, SUITE_SEED);
    let beyond = xs.iter().filter(|x| x.abs() > 1.0).count() as f64 / xs.len() as f64;
    assert_near(beyond, 0.5, 0.02, "P(|X| > 1)");
}

#[test]
fn the_cdf_matches_the_closed_form_across_the_body() {
    // `the_tails_are_heavy` checks the distribution at a single point, and a single point is a
    // weak grip on a sampler: the inverse-CDF form has several ways to be wrong that still land
    // half the mass outside the quartiles. `P(|X| < t) = (2/pi) atan(t)` is the closed form, and
    // checking it across the body pins the shape rather than one crossing.
    //
    // The tolerance is absolute, because the quantity is a probability and its sampling error
    // does not scale with its value: at 200 000 draws that error is 0.0011, so 0.004 is roughly
    // three and a half standard errors — a real bound, not a formality.
    let d = expect_accepted(Cauchy::new(0.0f64, 1.0f64), "standard Cauchy");
    let xs = draws::<f64, _>(&d, 200_000, SUITE_SEED);

    for t in [0.25f64, 0.5, 1.0, 2.0, 4.0] {
        let want = 2.0 / std::f64::consts::PI * t.atan();
        let got = xs.iter().filter(|x| x.abs() < t).count() as f64 / xs.len() as f64;
        assert!(
            (got - want).abs() < 0.004,
            "P(|X| < {t}): got {got}, closed form {want}, off by {}",
            (got - want).abs()
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Why there is no mean test
// ---------------------------------------------------------------------------------------------

#[test]
fn seed_instability_demonstrates_why_there_is_no_mean_test() {
    // Five seeds, five sample means. For a distribution with a mean these would agree to within
    // the statistical error; for this one they do not agree at all, because the sample mean of
    // Cauchy draws is itself Cauchy-distributed.
    let d = expect_accepted(Cauchy::new(0.0f64, 1.0f64), "standard Cauchy");
    let means: Vec<f64> = (0..5)
        .map(|k| {
            let xs = draws::<f64, _>(&d, N, SUITE_SEED + k);
            xs.iter().sum::<f64>() / xs.len() as f64
        })
        .collect();

    let lo = means.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = means.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let spread = hi - lo;

    // For comparison: 20 000 draws from a distribution with unit variance would put the sample
    // mean within about 0.007 of the truth, so five seeds would span roughly 0.02.
    println!("Cauchy sample means across five seeds: {means:?}");
    println!("  spread = {spread}");
    assert!(
        spread > 0.05,
        "the sample means across five seeds spanned only {spread:?} ({means:?}); if they had \
         agreed, a mean assertion would look sound and would still be meaningless"
    );
}

// ---------------------------------------------------------------------------------------------
// Every scalar
// ---------------------------------------------------------------------------------------------

#[test]
fn the_same_body_samples_every_scalar() {
    let d32 = expect_accepted(Cauchy::new(0.0f32, 1.0f32), "f32");
    let m32 = quantile(&sorted_draws::<f32, _>(&d32, N, SUITE_SEED), 0.5);
    assert!(m32.abs() < 0.1, "f32 median {m32}");

    let d64 = expect_accepted(Cauchy::new(0.0f64, 1.0f64), "f64");
    let m64 = quantile(&sorted_draws::<f64, _>(&d64, N, SUITE_SEED), 0.5);
    assert!(m64.abs() < 0.1, "f64 median {m64}");

    let d106 = expect_accepted(
        Cauchy::new(lift::<Float106>(0.0), lift::<Float106>(1.0)),
        "Float106",
    );
    let m106 = quantile(&sorted_draws::<Float106, _>(&d106, N, SUITE_SEED), 0.5);
    assert!(m106.abs() < 0.1, "Float106 median {m106}");
}

// ---------------------------------------------------------------------------------------------
// Parameters outside the support
// ---------------------------------------------------------------------------------------------

#[test]
fn a_non_positive_scale_is_refused() {
    expect_refused(Cauchy::new(0.0f64, 0.0f64), "a zero scale");
    expect_refused(Cauchy::new(0.0f64, -1.0f64), "a negative scale");
}

#[test]
fn a_non_finite_parameter_is_refused() {
    expect_refused(Cauchy::new(f64::NAN, 1.0f64), "a NaN location");
    expect_refused(Cauchy::new(0.0f64, f64::NAN), "a NaN scale");
    expect_refused(Cauchy::new(f64::INFINITY, 1.0f64), "an infinite location");
    expect_refused(Cauchy::new(0.0f64, f64::INFINITY), "an infinite scale");
}

#[test]
fn valid_parameters_are_kept_verbatim() {
    let d = expect_accepted(Cauchy::new(1.5f64, 2.5f64), "Cauchy(1.5, 2.5)");
    assert_eq!(d.location(), 1.5);
    assert_eq!(d.scale(), 2.5);
}
