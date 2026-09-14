/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared harness for the distribution suites.
//!
//! Every distribution added in this change is checked the same way: draw many times from a seeded
//! generator, reduce to a moment, and compare against the **closed form**. A sampler checked
//! against a reimplementation of itself checks neither, so nothing here computes an expected value
//! by a second route — the caller supplies the analytic answer.

use crate::StatsError;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, ToPrimitive};
use deep_causality_rand::{Distribution, Xoshiro256};

/// The seed every distribution suite draws from, so a failure is reproducible.
pub const SUITE_SEED: u64 = 0x5EED_2026;

/// `n` draws from a distribution, at the caller's scalar, lowered once for comparison.
pub fn draws<T, D>(d: &D, n: u64, seed: u64) -> Vec<f64>
where
    T: ToPrimitive,
    D: Distribution<T>,
{
    let mut g = Xoshiro256::from_seed(seed);
    (0..n)
        .map(|_| {
            d.sample(&mut g)
                .to_f64()
                .expect("every supported scalar lowers to f64")
        })
        .collect()
}

/// Sample mean and variance of `n` draws.
///
/// Both are accumulated in `f64` rather than the sampled scalar: the point of these tests is the
/// distribution, and a narrow scalar's accumulation saturates for reasons that have nothing to do
/// with the sampler. Where the accumulation itself is under test, the suite says so.
pub fn moments<T, D>(d: &D, n: u64, seed: u64) -> (f64, f64)
where
    T: ToPrimitive,
    D: Distribution<T>,
{
    let xs = draws::<T, D>(d, n, seed);
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    let var = xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / xs.len() as f64;
    (mean, var)
}

/// Sorted draws, for the distributions whose moments do not exist and whose quantiles do.
pub fn sorted_draws<T, D>(d: &D, n: u64, seed: u64) -> Vec<f64>
where
    T: ToPrimitive,
    D: Distribution<T>,
{
    let mut xs = draws::<T, D>(d, n, seed);
    xs.sort_by(|a, b| a.partial_cmp(b).expect("no draw is NaN"));
    xs
}

/// The `q`-th quantile of an already-sorted sample, by nearest rank.
pub fn quantile(sorted: &[f64], q: f64) -> f64 {
    assert!((0.0..=1.0).contains(&q), "a quantile lies in [0, 1]");
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx]
}

/// Assert a relative agreement with a closed form.
pub fn assert_near(got: f64, want: f64, tol: f64, what: &str) {
    let rel = if want == 0.0 {
        got.abs()
    } else {
        (got - want).abs() / want.abs()
    };
    assert!(
        rel <= tol,
        "{what}: got {got}, closed form {want}, relative error {rel} exceeds {tol}"
    );
}

/// Assert a constructor refused a parameter outside the distribution's support.
pub fn expect_refused<T>(got: Result<T, StatsError>, what: &str) {
    assert!(
        got.is_err(),
        "{what}: the constructor accepted a parameter the distribution is not defined for"
    );
}

/// Assert a constructor accepted a parameter inside the support.
pub fn expect_accepted<T>(got: Result<T, StatsError>, what: &str) -> T {
    match got {
        Ok(v) => v,
        Err(e) => panic!("{what}: the constructor refused a valid parameter: {e}"),
    }
}

/// The scalars every distribution is exercised at.
///
/// A sampler that works at one scalar has not been retrofitted. `BFloat16` is excluded from the
/// moment checks: its 8-bit significand saturates an accumulation long before the sample is large
/// enough to have a moment, which is the scalar's property and not the sampler's.
pub fn lift<T: FromPrimitive + RealField>(v: f64) -> T {
    T::from_f64(v).expect("a configuration literal converts to every supported scalar")
}

/// A generator whose words are all zero.
///
/// Several inverse-CDF draws take `ln(u)`, and a zero `u` gives an infinity. From a real generator
/// that happens with probability `2^-53` — one in nine quadrillion — so no sampling test reaches
/// it however many draws it takes. This one reaches it on the first draw.
///
/// A distribution that guards correctly redraws forever here, so a test asserts the guard is
/// *reached* rather than that a value comes back.
#[derive(Clone, Copy, Debug, Default)]
pub struct ZeroRng;

impl deep_causality_rand::RngCore for ZeroRng {
    fn next_u32(&mut self) -> u32 {
        0
    }
    fn next_u64(&mut self) -> u64 {
        0
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0);
    }
}

impl deep_causality_rand::Rng for ZeroRng {}

/// Assert a sampler does not terminate on an all-zero generator.
///
/// That is the observable for "this draw is taken from the open interval": a correct guard
/// redraws, and a redraw from `ZeroRng` is also zero, so the call never returns. The wait is
/// bounded and the thread is detached, so a defective sampler fails the assertion rather than
/// hanging the suite.
pub fn assert_guards_against_zero<F>(draw: F, what: &str)
where
    F: FnOnce() + Send + 'static,
{
    let handle = std::thread::spawn(draw);
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(
        !handle.is_finished(),
        "{what}: the sampler returned a value from an all-zero generator, so it admits a zero \
         into a logarithm"
    );
}

/// A generator whose words are all ones, giving a uniform draw as close to 1 as the scalar allows.
///
/// The counterpart to [`ZeroRng`]. Where a zero draw exercises a lower boundary, this exercises the
/// upper one: a cumulative scan that exhausts its weights, a rejection that must not loop, a
/// transform whose argument approaches its limit. Like the zero case it sits at probability
/// `2^-53` from a real generator and so cannot be reached by sampling.
#[derive(Clone, Copy, Debug, Default)]
pub struct MaxRng;

impl deep_causality_rand::RngCore for MaxRng {
    fn next_u32(&mut self) -> u32 {
        u32::MAX
    }
    fn next_u64(&mut self) -> u64 {
        u64::MAX
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0xFF);
    }
}

impl deep_causality_rand::Rng for MaxRng {}
