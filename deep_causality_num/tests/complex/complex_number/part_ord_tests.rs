/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::cmp::Ordering;
use deep_causality_num::{Complex, Float};

#[test]
fn test_partial_ord_equal() {
    let c1 = Complex::new(1.0f64, 2.0f64);
    let c2 = Complex::new(1.0f64, 2.0f64);
    assert_eq!(c1.partial_cmp(&c2), Some(Ordering::Equal));
}

#[test]
fn test_partial_ord_less_by_norm() {
    let c1 = Complex::new(1.0f64, 1.0f64); // norm sqrt(2)
    let c2 = Complex::new(2.0f64, 0.0f64); // norm 2
    assert_eq!(c1.partial_cmp(&c2), Some(Ordering::Less));
}

#[test]
fn test_partial_ord_greater_by_norm() {
    let c1 = Complex::new(2.0f64, 0.0f64); // norm 2
    let c2 = Complex::new(1.0f64, 1.0f64); // norm sqrt(2)
    assert_eq!(c1.partial_cmp(&c2), Some(Ordering::Greater));
}

#[test]
fn test_partial_ord_equal_norm_less_re() {
    let c1 = Complex::new(1.0f64, 2.0f64); // norm sqrt(5)
    let c2 = Complex::new(2.0f64, 1.0f64); // norm sqrt(5)
    assert_eq!(c1.partial_cmp(&c2), Some(Ordering::Less));
}

#[test]
fn test_partial_ord_equal_norm_greater_re() {
    let c1 = Complex::new(2.0f64, 1.0f64); // norm sqrt(5)
    let c2 = Complex::new(1.0f64, 2.0f64); // norm sqrt(5)
    assert_eq!(c1.partial_cmp(&c2), Some(Ordering::Greater));
}

#[test]
fn test_partial_ord_equal_norm_equal_re_less_im() {
    let c1 = Complex::new(1.0f64, 1.0f64); // norm sqrt(2)
    let c2 = Complex::new(1.0f64, 2.0f64); // norm sqrt(5)
    assert_eq!(c1.partial_cmp(&c2), Some(Ordering::Less));
}

#[test]
fn test_partial_ord_equal_norm_equal_re_greater_im() {
    let c1 = Complex::new(1.0f64, 2.0f64); // norm sqrt(5)
    let c2 = Complex::new(1.0f64, 1.0f64); // norm sqrt(2)
    assert_eq!(c1.partial_cmp(&c2), Some(Ordering::Greater));
}

#[test]
fn test_partial_ord_nan_re() {
    let c1 = Complex::new(f64::nan(), 2.0f64);
    let c2 = Complex::new(1.0f64, 2.0f64);
    assert_eq!(c1.partial_cmp(&c2), None);
}

#[test]
fn test_partial_ord_nan_im() {
    let c1 = Complex::new(1.0f64, f64::nan());
    let c2 = Complex::new(1.0f64, 2.0f64);
    assert_eq!(c1.partial_cmp(&c2), None);
}

#[test]
fn test_partial_ord_nan_both() {
    let c1 = Complex::new(f64::nan(), f64::nan());
    let c2 = Complex::new(1.0f64, 2.0f64);
    assert_eq!(c1.partial_cmp(&c2), None);
}
