/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The wake-probe frequency reduction (mean-crossing counting → Strouhal number),
//! pinned on synthetic signals so the math is verified independently of a shedding sim.

use deep_causality_cfd::{dominant_frequency, strouhal_number};

/// A clean sine at a known frequency is recovered to within the mean-crossing
/// quantization (one crossing ≈ half a sample period).
#[test]
fn dominant_frequency_recovers_a_known_sine() {
    let f = 2.0_f64; // Hz
    let dt = 0.001_f64;
    let n = 4000; // 4 s ⇒ 8 full periods
    let signal: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * f * i as f64 * dt).sin())
        .collect();

    let estimate = dominant_frequency(&signal, dt);
    assert!(
        (estimate - f).abs() < 0.05,
        "recovered frequency {estimate} far from {f}"
    );
}

/// `St = f·L / U`: a 2 Hz signal with diameter 0.5 and free-stream 4 gives St = 0.25.
#[test]
fn strouhal_number_composes_f_length_speed() {
    let f = 2.0_f64;
    let dt = 0.001_f64;
    let n = 4000;
    let signal: Vec<f64> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * f * i as f64 * dt).sin())
        .collect();

    let st = strouhal_number(&signal, dt, 0.5, 4.0);
    let expected = f * 0.5 / 4.0; // 0.25
    assert!((st - expected).abs() < 0.02, "St {st} far from {expected}");
}

/// A monotone (non-oscillating) signal has no detectable frequency.
#[test]
fn dominant_frequency_of_a_ramp_is_zero() {
    let dt = 0.01_f64;
    let signal: Vec<f64> = (0..200).map(|i| i as f64).collect();
    assert_eq!(dominant_frequency(&signal, dt), 0.0);
    assert_eq!(strouhal_number(&signal, dt, 1.0, 1.0), 0.0);
}

/// Degenerate inputs (too few samples, non-positive dt or speed) return zero, not NaN.
#[test]
fn frequency_guards_degenerate_inputs() {
    assert_eq!(dominant_frequency(&[1.0_f64, 2.0], 0.01), 0.0);
    assert_eq!(dominant_frequency(&[1.0_f64, 2.0, 3.0], 0.0), 0.0);
    let sine: Vec<f64> = (0..400).map(|i| (0.1 * i as f64).sin()).collect();
    assert_eq!(strouhal_number(&sine, 0.01, 1.0, 0.0), 0.0);
}
