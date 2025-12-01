/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::utils_tests::utils_octonion_tests;
use deep_causality_num::{Float, Octonion};

#[test]
fn test_octonion_conjugate() {
    let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let conj_o = o.conjugate();
    utils_octonion_tests::assert_octonion_approx_eq(
        conj_o,
        Octonion::new(1.0, -2.0, -3.0, -4.0, -5.0, -6.0, -7.0, -8.0),
        1e-9,
    );
}

#[test]
fn test_octonion_norm_sqr() {
    let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let expected_norm_sqr = 1.0f64.powi(2)
        + 2.0f64.powi(2)
        + 3.0f64.powi(2)
        + 4.0f64.powi(2)
        + 5.0f64.powi(2)
        + 6.0f64.powi(2)
        + 7.0f64.powi(2)
        + 8.0f64.powi(2);
    assert!((o.norm_sqr() - expected_norm_sqr).abs() < 1e-9);
}

#[test]
fn test_octonion_norm() {
    let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let expected_norm = (1.0f64.powi(2)
        + 2.0f64.powi(2)
        + 3.0f64.powi(2)
        + 4.0f64.powi(2)
        + 5.0f64.powi(2)
        + 6.0f64.powi(2)
        + 7.0f64.powi(2)
        + 8.0f64.powi(2))
    .sqrt();
    assert!((o.norm() - expected_norm).abs() < 1e-9);
}

#[test]
fn test_octonion_normalize() {
    let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let norm = o.norm();
    let normalized_o = o.normalize();
    utils_octonion_tests::assert_octonion_approx_eq(
        normalized_o,
        Octonion::new(
            1.0 / norm,
            2.0 / norm,
            3.0 / norm,
            4.0 / norm,
            5.0 / norm,
            6.0 / norm,
            7.0 / norm,
            8.0 / norm,
        ),
        1e-9,
    );

    // Test normalize zero octonion
    let zero_o = Octonion::<f64>::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    utils_octonion_tests::assert_octonion_approx_eq(zero_o.normalize(), zero_o, 1e-9);
}

#[test]
fn test_octonion_inverse() {
    let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let norm_sqr = o.norm_sqr();
    let inv_o = o.inverse();
    utils_octonion_tests::assert_octonion_approx_eq(
        inv_o,
        Octonion::new(
            1.0 / norm_sqr,
            -2.0 / norm_sqr,
            -3.0 / norm_sqr,
            -4.0 / norm_sqr,
            -5.0 / norm_sqr,
            -6.0 / norm_sqr,
            -7.0 / norm_sqr,
            -8.0 / norm_sqr,
        ),
        1e-9,
    );

    // Test inverse of zero octonion
    let zero_o = Octonion::<f64>::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let inv_zero_o = zero_o.inverse();
    assert!(inv_zero_o.s.is_nan());
    assert!(inv_zero_o.e1.is_nan());
    assert!(inv_zero_o.e7.is_nan());
}

#[test]
fn test_octonion_dot() {
    let o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let o2 = Octonion::new(9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
    let expected_dot = 1.0 * 9.0
        + 2.0 * 10.0
        + 3.0 * 11.0
        + 4.0 * 12.0
        + 5.0 * 13.0
        + 6.0 * 14.0
        + 7.0 * 15.0
        + 8.0 * 16.0;
    assert!((o1.dot(&o2) - expected_dot).abs() < 1e-9);
}
