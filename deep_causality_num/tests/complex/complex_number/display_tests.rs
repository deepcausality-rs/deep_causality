/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;

#[test]
fn test_display_complex_positive_im() {
    let c = Complex::new(1.0f64, 2.0f64);
    assert_eq!(format!("{}", c), "1+2i");
}

#[test]
fn test_display_complex_negative_im() {
    let c = Complex::new(1.0f64, -2.0f64);
    assert_eq!(format!("{}", c), "1-2.0i");
}

#[test]
fn test_display_complex_zero_im() {
    let c = Complex::new(1.0f64, 0.0f64);
    assert_eq!(format!("{}", c), "1+0i");
}

#[test]
fn test_display_complex_zero_re() {
    let c = Complex::new(0.0f64, 2.0f64);
    assert_eq!(format!("{}", c), "0+2i");
}

#[test]
fn test_display_complex_zero_re_negative_im() {
    let c = Complex::new(0.0f64, -2.0f64);
    assert_eq!(format!("{}", c), "0-2.0i");
}
