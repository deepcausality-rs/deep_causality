/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A published value with its standard error, carried onto a fringe as effective draws.
//!
//! The reference numbers are the design note's read-out arithmetic for a rubidium gravimeter at
//! mid-fringe: `C = 0.5`, `k_eff = 1.6106 × 10⁷ rad/m`, `T = 100 ms`, so the slope is
//! `C · k_eff · T² / 2 = 40 265` per m/s². −5 µGal reads 0.498, and a sensitivity of
//! 24 µGal/√Hz is 1/(C k_eff T² S)² ≈ 2677 effective draws in one second.

use deep_causality_quantum::{Fringe, QuantumErrorEnum};

const SLOPE: f64 = 0.5 * 1.6106e7 * 0.1 * 0.1 / 2.0;

fn mid_fringe() -> Fringe<f64> {
    Fringe::new(0.5, SLOPE).unwrap()
}

#[test]
fn test_the_fringe_keeps_its_operating_point_and_slope() {
    let f = mid_fringe();
    assert_eq!((f.operating_point(), f.slope()), (0.5, SLOPE));
    assert!(Fringe::new(0.3, -2.0).is_ok(), "a falling fringe reads too");
}

#[test]
fn test_a_fringe_that_reads_nothing_is_refused() {
    for op in [0.0, 1.0] {
        assert!(matches!(
            Fringe::new(op, 1.0).unwrap_err().0,
            QuantumErrorEnum::CalculationError(_)
        ));
    }
    assert!(matches!(
        Fringe::new(0.5, 0.0).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
    for (op, slope) in [(f64::NAN, 1.0), (0.5, f64::INFINITY)] {
        assert!(matches!(
            Fringe::new(op, slope).unwrap_err().0,
            QuantumErrorEnum::NonFiniteValue(_)
        ));
    }
}

#[test]
fn test_the_note_numbers() {
    let f = mid_fringe();
    // −5 µGal moves the read-out from 0.500 to 0.498.
    let d = f.effective_draws(-5.0e-8, 2.4e-7).unwrap();
    assert!(
        (d.probability() - 0.498).abs() < 5.0e-5,
        "{}",
        d.probability()
    );
    // 24 µGal over one second at the operating point: 1/(C k_eff T² S)² draws.
    let d = f.effective_draws(0.0, 2.4e-7).unwrap();
    let expected = 1.0 / (0.5 * 1.6106e7 * 0.01 * 2.4e-7_f64).powi(2);
    assert!((d.draws() / expected - 1.0).abs() < 1e-12);
    assert_eq!(d.read_out().unwrap().shots(), 2677);
}

#[test]
fn test_a_published_value_converts_to_draws_and_back() {
    let eps = f64::EPSILON;
    for f in [mid_fringe(), Fringe::new(0.2, -3.0).unwrap()] {
        for (value, sigma) in [(-5.0e-8, 2.4e-7), (1.3e-6, 4.0e-8), (0.0, 1.0e-9)] {
            let (value, sigma) = if f.slope() < 0.0 {
                (value * 1.0e4, sigma * 1.0e4)
            } else {
                (value, sigma)
            };
            let d = f.effective_draws(value, sigma).unwrap();
            let (v, s) = f.published(&d);
            // p = op + slope·v rounds once at the scale of p, which bounds the value's error.
            assert!(
                (v - value).abs() <= 4.0 * eps / f.slope().abs(),
                "{v} vs {value}"
            );
            assert!((s / sigma - 1.0).abs() <= 8.0 * eps, "{s} vs {sigma}");
            // The read-out carries the same probability and width.
            let r = d.read_out().unwrap();
            assert_eq!(r.estimate(), d.probability());
            assert_eq!(r.standard_error(), d.standard_error());
        }
    }
}

#[test]
fn test_a_value_the_fringe_cannot_read_is_refused() {
    let f = mid_fringe();
    // 0.5 / SLOPE puts the read-out at 1, and minus it at 0.
    for value in [0.5 / SLOPE, -0.5 / SLOPE] {
        assert!(matches!(
            f.effective_draws(value, 1.0e-8).unwrap_err().0,
            QuantumErrorEnum::CalculationError(_)
        ));
    }
    for sigma in [0.0, -1.0e-8] {
        assert!(matches!(
            f.effective_draws(0.0, sigma).unwrap_err().0,
            QuantumErrorEnum::CalculationError(_)
        ));
    }
    for (value, sigma) in [(f64::NAN, 1.0e-8), (0.0, f64::INFINITY)] {
        assert!(matches!(
            f.effective_draws(value, sigma).unwrap_err().0,
            QuantumErrorEnum::NonFiniteValue(_)
        ));
    }
}
