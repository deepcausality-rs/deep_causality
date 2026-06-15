/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Frequency analysis of a wake-probe time series — the Strouhal reduction.
//!
//! A vortex-shedding wake oscillates the transverse velocity at a fixed downstream point
//! (the [`Observe::probe`](crate::Observe::probe) signal). Counting the mean-crossings of
//! that signal recovers the dominant shedding frequency without an FFT, which is robust on
//! the short, evenly-sampled series a march produces. The Strouhal number then follows as
//! `St = f·L / U`.

use crate::types::CfdScalar;

/// The dominant frequency of an evenly-sampled signal by **mean-crossing counting**:
/// each pair of consecutive crossings of the signal mean spans one half-period, so
/// `f = (crossings / 2) / T` with `T = (n − 1)·dt` the record length. Returns `0` when
/// fewer than two crossings are seen (no detectable oscillation over the record).
///
/// Robust for the short, low-noise records a march yields; it does not resolve multiple
/// spectral peaks (a single dominant tone is assumed, as in a clean shedding wake).
pub fn dominant_frequency<R: CfdScalar>(signal: &[R], dt: R) -> R {
    if signal.len() < 3 || dt <= R::zero() {
        return R::zero();
    }
    let n = signal.len();
    let mean = signal.iter().fold(R::zero(), |acc, x| acc + *x)
        / R::from_usize(n).expect("the sample count lifts into every real field");

    let mut crossings = 0usize;
    let mut prev = signal[0] - mean;
    for &s in &signal[1..] {
        let cur = s - mean;
        // A sign change (strictly across zero) is one mean-crossing.
        if (prev < R::zero() && cur >= R::zero()) || (prev > R::zero() && cur <= R::zero()) {
            crossings += 1;
        }
        prev = cur;
    }
    if crossings < 2 {
        return R::zero();
    }

    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    let periods = R::from_usize(crossings).expect("the crossing count lifts into R") * half;
    let total_time = R::from_usize(n - 1).expect("the sample count lifts into R") * dt;
    periods / total_time
}

/// The Strouhal number `St = f·L / U` of a wake-probe `signal` sampled every `dt`, with
/// characteristic length `length` (the body diameter) and free-stream speed `u_ref`.
/// Returns `0` when no oscillation is detected (see [`dominant_frequency`]).
pub fn strouhal_number<R: CfdScalar>(signal: &[R], dt: R, length: R, u_ref: R) -> R {
    if u_ref <= R::zero() {
        return R::zero();
    }
    dominant_frequency(signal, dt) * length / u_ref
}
