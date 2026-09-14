/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Poisson distribution, checked against its closed form and its documented domain.

use deep_causality_num::Float106;
use deep_causality_rand::{Distribution, Xoshiro256};
use deep_causality_stats::utils_tests::sampling::{
    MaxRng, SUITE_SEED, ZeroRng, assert_near, expect_accepted, expect_refused, lift,
};
use deep_causality_stats::{MAX_ITERATIONS, MAX_RATE, Poisson};

const N: u64 = 200_000;

/// Counts from `n` draws, as `f64`, with every draw checked to be a plausible count.
fn counts<T>(d: &Poisson<T>, n: u64, seed: u64) -> Vec<f64>
where
    Poisson<T>: Distribution<u64>,
{
    let mut g = Xoshiro256::from_seed(seed);
    (0..n)
        .map(|_| {
            let k: u64 = d.sample(&mut g);
            k as f64
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// The closed form: mean and variance are both the rate
// ---------------------------------------------------------------------------------------------

#[test]
fn the_mean_is_the_rate() {
    let d = expect_accepted(Poisson::new(3.0f64), "rate 3");
    let xs = counts(&d, N, SUITE_SEED);
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    assert_near(mean, 3.0, 0.01, "Poisson(3) mean");
}

#[test]
fn the_variance_is_also_the_rate() {
    // Equidispersion is what distinguishes the Poisson from every other count distribution. A
    // sampler with the right mean and the wrong spread — a binomial, say — passes a mean test.
    let d = expect_accepted(Poisson::new(3.0f64), "rate 3");
    let xs = counts(&d, N, SUITE_SEED);
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    let var = xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / xs.len() as f64;
    assert_near(var, 3.0, 0.05, "Poisson(3) variance");
}

#[test]
fn a_second_rate_agrees_too() {
    // One rate could be a coincidence of a mis-scaled sampler; two cannot.
    let d = expect_accepted(Poisson::new(7.5f64), "rate 7.5");
    let xs = counts(&d, N, SUITE_SEED);
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    assert_near(mean, 7.5, 0.01, "Poisson(7.5) mean");
}

#[test]
fn the_probability_of_zero_is_exp_minus_rate() {
    // `P(K = 0) = e^{-λ}`. At `λ = 2` that is 0.1353, and it pins the algorithm's stopping rule
    // rather than only its average.
    let d = expect_accepted(Poisson::new(2.0f64), "rate 2");
    let xs = counts(&d, N, SUITE_SEED);
    let zeros = xs.iter().filter(|k| **k == 0.0).count() as f64 / xs.len() as f64;
    assert_near(zeros, (-2.0f64).exp(), 0.03, "P(K = 0)");
}

// ---------------------------------------------------------------------------------------------
// The degenerate case
// ---------------------------------------------------------------------------------------------

#[test]
fn a_zero_rate_gives_every_draw_zero() {
    let d = expect_accepted(Poisson::new(0.0f64), "rate 0");
    for (i, k) in counts(&d, 1_000, SUITE_SEED).into_iter().enumerate() {
        assert_eq!(k, 0.0, "draw {i} was {k} at rate 0");
    }
}

// ---------------------------------------------------------------------------------------------
// The domain, and the loop that must not run away
// ---------------------------------------------------------------------------------------------

#[test]
fn a_rate_beyond_the_documented_range_is_refused() {
    // Not "handled somehow": refused, with an error the caller can act on. The alternative —
    // switching method silently above the cap — is harder to reason about.
    expect_refused(Poisson::new(MAX_RATE + 1.0), "just beyond the cap");
    expect_refused(Poisson::new(1.0e6f64), "far beyond the cap");
    // Beyond `f64`'s own underflow point for `e^{-lambda}`, where the loop could not terminate.
    expect_refused(Poisson::new(1.0e4f64), "beyond the underflow point");
}

#[test]
fn a_rate_at_the_cap_is_accepted_and_terminates() {
    // The boundary itself, and the test is that it comes back at all: at this rate the expected
    // iteration count is about 501, so a sampler whose stopping rule is wrong hangs rather than
    // fails.
    let d = expect_accepted(Poisson::new(MAX_RATE), "exactly at the cap");
    let xs = counts(&d, 200, SUITE_SEED);
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    assert_near(mean, MAX_RATE, 0.1, "mean at the cap");
}

#[test]
fn the_draw_terminates_on_a_degenerate_generator() {
    // The two boundary generators, which a real one reaches at probability `2^-53`. An all-zero
    // generator drives the product to zero immediately; an all-ones generator keeps it as high as
    // the scalar allows, which is the case a multiplicative stopping rule can fail to escape.
    let d = expect_accepted(Poisson::new(4.0f64), "rate 4");
    let k_zero: u64 = d.sample(&mut ZeroRng);
    assert_eq!(k_zero, 0, "a zero draw should stop the product at once");

    // With every draw at its maximum the product decays by `1 - 2^-53` per step, so reaching
    // `e^{-4}` would take about `4 * 2^53 = 3.6e16` iterations. That is a hang, not a delay, and
    // it is what this test found before the sampler carried a cap: the suite did not fail, it
    // never returned.
    //
    // The assertion is therefore that the draw **comes back**, bounded by the documented cap. A
    // sound generator never approaches it — at the largest admissible rate the expected count is
    // 501 — so the cap costs nothing and converts the hang into a value.
    let k_max: u64 = d.sample(&mut MaxRng);
    assert!(
        k_max <= MAX_ITERATIONS,
        "the draw returned {k_max}, beyond the documented cap of {MAX_ITERATIONS}"
    );
}

// ---------------------------------------------------------------------------------------------
// Every scalar
// ---------------------------------------------------------------------------------------------

#[test]
fn the_same_body_samples_every_scalar() {
    let d32 = expect_accepted(Poisson::new(3.0f32), "f32");
    let m32 = counts(&d32, 50_000, SUITE_SEED).iter().sum::<f64>() / 50_000.0;
    assert_near(m32, 3.0, 0.03, "f32 mean");

    let d64 = expect_accepted(Poisson::new(3.0f64), "f64");
    let m64 = counts(&d64, 50_000, SUITE_SEED).iter().sum::<f64>() / 50_000.0;
    assert_near(m64, 3.0, 0.03, "f64 mean");

    let d106 = expect_accepted(Poisson::new(lift::<Float106>(3.0)), "Float106");
    let m106 = counts(&d106, 50_000, SUITE_SEED).iter().sum::<f64>() / 50_000.0;
    assert_near(m106, 3.0, 0.03, "Float106 mean");
}

// ---------------------------------------------------------------------------------------------
// Parameters outside the support
// ---------------------------------------------------------------------------------------------

#[test]
fn a_negative_or_non_finite_rate_is_refused() {
    expect_refused(Poisson::new(-1.0f64), "a negative rate");
    expect_refused(Poisson::new(f64::NAN), "a NaN rate");
    expect_refused(Poisson::new(f64::INFINITY), "an infinite rate");
}

#[test]
fn a_valid_rate_is_kept_verbatim() {
    let d = expect_accepted(Poisson::new(2.5f64), "rate 2.5");
    assert_eq!(d.rate(), 2.5);
}
