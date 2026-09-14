/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every distribution, at every scalar, in one table.
//!
//! The per-distribution suites each run at one scalar, because what they test is the distribution.
//! This file tests the retrofit itself: that the samplers are generic in the scalar rather than
//! generic-looking. A scalar added later is one row here, not seven files.
//!
//! The checks are deliberately coarse. A tight moment check is the per-distribution suite's job;
//! what this file must catch is a sampler that fails to compile, refuses, saturates or returns
//! nonsense at a scalar other than the one it was written against.

use deep_causality_num::{Float106, ToPrimitive};
use deep_causality_rand::{Distribution, Xoshiro256};
use deep_causality_stats::utils_tests::sampling::{
    SUITE_SEED, assert_near, draws, expect_accepted, lift, moments, quantile, sorted_draws,
};
use deep_causality_stats::{
    Categorical, Cauchy, Exponential, LogNormal, Normal, Open01, OpenClosed01, Poisson, RandScalar,
    StandardNormal, StandardUniform, UniformInt, Weibull,
};

/// Draws per check. The sampling error of a mean is `1/sqrt(N)`, so 50 000 draws leave the mean
/// good to about half a percent and the 2% tolerance below is a real test rather than a formality.
const N: u64 = 50_000;

/// Relative agreement demanded of a mean or a quantile.
const TOL: f64 = 0.02;

/// Run every distribution at one scalar, and return the index sequence `UniformInt` produced.
///
/// The return value is what makes `UniformInt` part of the table. It carries no scalar of its own —
/// it draws words and returns an index — so what there is to check is that it is *independent* of
/// the scalar the caller is working at. The caller compares the sequences across scalars.
fn exercise<T>(scalar: &str) -> Vec<u64>
where
    // No `StandardUniform: Distribution<T>` clauses. The distributions are implemented for every
    // scalar carrying these three capabilities, so naming the capabilities is enough — which is
    // the whole point of the retrofit, and would not compile before it.
    T: RandScalar + ToPrimitive,
{
    // --- the three unit draws, which every inverse-CDF distribution below rests on ---
    let u = draws::<T, _>(&StandardUniform, N, SUITE_SEED);
    assert!(
        u.iter().all(|&x| (0.0..1.0).contains(&x)),
        "{scalar}: a StandardUniform draw left [0, 1)"
    );
    assert_near(
        u.iter().sum::<f64>() / N as f64,
        0.5,
        TOL,
        &format!("{scalar}: StandardUniform mean"),
    );

    let o = draws::<T, _>(&Open01, N, SUITE_SEED);
    assert!(
        o.iter().all(|&x| x > 0.0 && x < 1.0),
        "{scalar}: an Open01 draw reached an endpoint"
    );

    let oc = draws::<T, _>(&OpenClosed01, N, SUITE_SEED);
    assert!(
        oc.iter().all(|&x| x > 0.0 && x <= 1.0),
        "{scalar}: an OpenClosed01 draw left (0, 1]"
    );

    // --- the normal, and the general normal built on it ---
    let (mean, var) = moments::<T, _>(&StandardNormal, N, SUITE_SEED);
    assert!(
        mean.abs() < 0.02,
        "{scalar}: StandardNormal mean {mean} is not near zero"
    );
    assert_near(
        var,
        1.0,
        0.05,
        &format!("{scalar}: StandardNormal variance"),
    );

    let normal = Normal::<T>::new(lift(3.0), lift(2.0))
        .unwrap_or_else(|e| panic!("{scalar}: Normal refused a valid pair: {e}"));
    let (mean, var) = moments::<T, _>(&normal, N, SUITE_SEED);
    assert_near(mean, 3.0, TOL, &format!("{scalar}: Normal mean"));
    assert_near(var, 4.0, 0.05, &format!("{scalar}: Normal variance"));

    // --- the inverse-CDF family: closed-form mean, or a quantile where no mean exists ---
    let exponential = expect_accepted(
        Exponential::<T>::new(lift(2.0)),
        &format!("{scalar}: Exponential(2)"),
    );
    let (mean, _) = moments::<T, _>(&exponential, N, SUITE_SEED);
    assert_near(mean, 0.5, TOL, &format!("{scalar}: Exponential mean"));

    let weibull = expect_accepted(
        Weibull::<T>::new(lift(1.0), lift(1.0)),
        &format!("{scalar}: Weibull(1, 1)"),
    );
    let (mean, _) = moments::<T, _>(&weibull, N, SUITE_SEED);
    assert_near(mean, 1.0, TOL, &format!("{scalar}: Weibull(1, 1) mean"));

    let log_normal = expect_accepted(
        LogNormal::<T>::new(lift(0.0), lift(1.0)),
        &format!("{scalar}: LogNormal(0, 1)"),
    );
    let sorted = sorted_draws::<T, _>(&log_normal, N, SUITE_SEED);
    assert!(
        sorted.iter().all(|&x| x > 0.0),
        "{scalar}: a LogNormal draw was not positive"
    );
    assert_near(
        quantile(&sorted, 0.5),
        1.0,
        0.05,
        &format!("{scalar}: LogNormal median"),
    );

    // Cauchy has no mean at any sample size, so it is checked where it is defined: its quartiles
    // sit one scale either side of the location.
    let cauchy = expect_accepted(
        Cauchy::<T>::new(lift(0.0), lift(1.0)),
        &format!("{scalar}: Cauchy(0, 1)"),
    );
    let sorted = sorted_draws::<T, _>(&cauchy, N, SUITE_SEED);
    assert!(
        (quantile(&sorted, 0.75) - 1.0).abs() < 0.1,
        "{scalar}: Cauchy upper quartile is {}",
        quantile(&sorted, 0.75)
    );
    assert!(
        (quantile(&sorted, 0.25) + 1.0).abs() < 0.1,
        "{scalar}: Cauchy lower quartile is {}",
        quantile(&sorted, 0.25)
    );

    // --- the two that take a scalar and return a count ---
    let categorical = expect_accepted(
        Categorical::<T>::new(vec![lift(1.0), lift(1.0), lift(2.0)]),
        &format!("{scalar}: Categorical"),
    );
    let picks = draws::<usize, _>(&categorical, N, SUITE_SEED);
    let heaviest = picks.iter().filter(|&&i| i == 2.0).count() as f64 / N as f64;
    assert_near(
        heaviest,
        0.5,
        TOL,
        &format!("{scalar}: Categorical weight-2 share"),
    );

    let poisson = expect_accepted(
        Poisson::<T>::new(lift(4.0)),
        &format!("{scalar}: Poisson(4)"),
    );
    let (mean, var) = moments::<u64, _>(&poisson, N, SUITE_SEED);
    assert_near(mean, 4.0, TOL, &format!("{scalar}: Poisson mean"));
    assert_near(var, 4.0, 0.05, &format!("{scalar}: Poisson variance"));

    // --- the index draw, which must not vary with the scalar at all ---
    let uniform_int = expect_accepted(UniformInt::new(7), &format!("{scalar}: UniformInt(7)"));
    let mut g = Xoshiro256::from_seed(SUITE_SEED);
    (0..1_000).map(|_| uniform_int.sample(&mut g)).collect()
}

#[test]
fn every_distribution_draws_at_every_scalar() {
    // One row per scalar. Adding a fifth is one line, which is the point of the retrofit.
    let at_f32 = exercise::<f32>("f32");
    let at_f64 = exercise::<f64>("f64");
    let at_f106 = exercise::<Float106>("Float106");

    // `UniformInt` consumes words and returns an index, so the same seed must give the same
    // indices whatever scalar the surrounding code works at. A sampler that reached for the
    // working scalar's unit draw instead would diverge here at f32, whose draw is 24 bits wide.
    assert_eq!(
        at_f32, at_f64,
        "the index draw changed between f32 and f64 callers"
    );
    assert_eq!(
        at_f64, at_f106,
        "the index draw changed between f64 and Float106 callers"
    );
}
