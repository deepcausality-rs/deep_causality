/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::utils_tests::utils_complex_tests;
use deep_causality_num::{Complex, ComplexNumber};
use std::f64::consts::PI;

#[test]
fn test_complex_norm_sqr() {
    let c = Complex::new(3.0, 4.0);
    assert_eq!(c.norm_sqr(), 25.0);
}

#[test]
fn test_complex_norm() {
    let c = Complex::new(3.0, 4.0);
    assert_eq!(c.norm(), 5.0);
}

#[test]
fn test_complex_arg() {
    let c = Complex::new(1.0, 1.0);
    utils_complex_tests::assert_approx_eq(c.arg(), PI / 4.0, 1e-9);
}

#[test]
fn test_complex_conj() {
    let c = Complex::new(1.0, 2.0);
    let conj_c = c.conj();
    assert_eq!(conj_c.re(), 1.0);
    assert_eq!(conj_c.im(), -2.0);
}
