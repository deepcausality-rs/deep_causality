/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for ComplexField trait implementation.

use deep_causality_num::{Complex, Complex64, ComplexField};
use std::f64::consts::PI;

const EPSILON: f64 = 1e-9;

// =============================================================================
// real() and imag() Tests
// =============================================================================

#[test]
fn test_real() {
    let z = Complex64::new(3.0, 4.0);
    assert!((ComplexField::real(&z) - 3.0).abs() < EPSILON);
}

#[test]
fn test_imag() {
    let z = Complex64::new(3.0, 4.0);
    assert!((ComplexField::imag(&z) - 4.0).abs() < EPSILON);
}

#[test]
fn test_real_pure_imaginary() {
    let z = Complex64::new(0.0, 5.0);
    assert!((ComplexField::real(&z) - 0.0).abs() < EPSILON);
}

#[test]
fn test_imag_pure_real() {
    let z = Complex64::new(5.0, 0.0);
    assert!((ComplexField::imag(&z) - 0.0).abs() < EPSILON);
}

// =============================================================================
// norm() Tests
// =============================================================================

#[test]
fn test_norm() {
    let z = Complex64::new(3.0, 4.0);
    // |3+4i| = sqrt(9+16) = sqrt(25) = 5
    assert!((ComplexField::norm(&z) - 5.0).abs() < EPSILON);
}

#[test]
fn test_norm_pure_real() {
    let z = Complex64::new(5.0, 0.0);
    assert!((ComplexField::norm(&z) - 5.0).abs() < EPSILON);
}

#[test]
fn test_norm_pure_imaginary() {
    let z = Complex64::new(0.0, 7.0);
    assert!((ComplexField::norm(&z) - 7.0).abs() < EPSILON);
}

#[test]
fn test_norm_zero() {
    let z = Complex64::new(0.0, 0.0);
    assert!((ComplexField::norm(&z) - 0.0).abs() < EPSILON);
}

#[test]
fn test_norm_negative() {
    let z = Complex64::new(-3.0, -4.0);
    assert!((ComplexField::norm(&z) - 5.0).abs() < EPSILON);
}

// =============================================================================
// arg() Tests
// =============================================================================

#[test]
fn test_arg_first_quadrant() {
    let z = Complex64::new(1.0, 1.0);
    // arg(1+i) = π/4
    assert!((ComplexField::arg(&z) - PI / 4.0).abs() < EPSILON);
}

#[test]
fn test_arg_positive_real() {
    let z = Complex64::new(5.0, 0.0);
    // arg on positive real axis = 0
    assert!((ComplexField::arg(&z) - 0.0).abs() < EPSILON);
}

#[test]
fn test_arg_positive_imaginary() {
    let z = Complex64::new(0.0, 5.0);
    // arg on positive imaginary axis = π/2
    assert!((ComplexField::arg(&z) - PI / 2.0).abs() < EPSILON);
}

#[test]
fn test_arg_negative_real() {
    let z = Complex64::new(-5.0, 0.0);
    // arg on negative real axis = π
    assert!((ComplexField::arg(&z) - PI).abs() < EPSILON);
}

#[test]
fn test_arg_negative_imaginary() {
    let z = Complex64::new(0.0, -5.0);
    // arg on negative imaginary axis = -π/2
    assert!((ComplexField::arg(&z) - (-PI / 2.0)).abs() < EPSILON);
}

#[test]
fn test_arg_second_quadrant() {
    let z = Complex64::new(-1.0, 1.0);
    // arg(-1+i) = 3π/4
    assert!((ComplexField::arg(&z) - 3.0 * PI / 4.0).abs() < EPSILON);
}

#[test]
fn test_arg_third_quadrant() {
    let z = Complex64::new(-1.0, -1.0);
    // arg(-1-i) = -3π/4
    assert!((ComplexField::arg(&z) - (-3.0 * PI / 4.0)).abs() < EPSILON);
}

#[test]
fn test_arg_fourth_quadrant() {
    let z = Complex64::new(1.0, -1.0);
    // arg(1-i) = -π/4
    assert!((ComplexField::arg(&z) - (-PI / 4.0)).abs() < EPSILON);
}

// =============================================================================
// from_re_im() Tests
// =============================================================================

#[test]
fn test_from_re_im() {
    let z: Complex<f64> = ComplexField::from_re_im(3.0, 4.0);
    assert!((z.re - 3.0).abs() < EPSILON);
    assert!((z.im - 4.0).abs() < EPSILON);
}

#[test]
fn test_from_re_im_zero() {
    let z: Complex<f64> = ComplexField::from_re_im(0.0, 0.0);
    assert!((z.re - 0.0).abs() < EPSILON);
    assert!((z.im - 0.0).abs() < EPSILON);
}

#[test]
fn test_from_re_im_negative() {
    let z: Complex<f64> = ComplexField::from_re_im(-5.0, -7.0);
    assert!((z.re - (-5.0)).abs() < EPSILON);
    assert!((z.im - (-7.0)).abs() < EPSILON);
}

// =============================================================================
// from_polar() Tests
// =============================================================================

#[test]
fn test_from_polar_unit_circle() {
    // r=1, θ=π/4 => 1*(cos(π/4) + i*sin(π/4)) = (√2/2) + i(√2/2)
    let z: Complex<f64> = ComplexField::from_polar(1.0, PI / 4.0);
    let expected = 2.0_f64.sqrt() / 2.0;
    assert!((z.re - expected).abs() < EPSILON);
    assert!((z.im - expected).abs() < EPSILON);
}

#[test]
fn test_from_polar_radius_two() {
    // r=2, θ=0 => 2*(cos(0) + i*sin(0)) = 2 + 0i
    let z: Complex<f64> = ComplexField::from_polar(2.0, 0.0);
    assert!((z.re - 2.0).abs() < EPSILON);
    assert!((z.im - 0.0).abs() < EPSILON);
}

#[test]
fn test_from_polar_pi() {
    // r=1, θ=π => cos(π) + i*sin(π) = -1 + 0i
    let z: Complex<f64> = ComplexField::from_polar(1.0, PI);
    assert!((z.re - (-1.0)).abs() < EPSILON);
    assert!(z.im.abs() < EPSILON);
}

#[test]
fn test_from_polar_half_pi() {
    // r=3, θ=π/2 => 3*(cos(π/2) + i*sin(π/2)) = 0 + 3i
    let z: Complex<f64> = ComplexField::from_polar(3.0, PI / 2.0);
    assert!(z.re.abs() < EPSILON);
    assert!((z.im - 3.0).abs() < EPSILON);
}

#[test]
fn test_from_polar_zero_radius() {
    // r=0 always gives origin
    let z: Complex<f64> = ComplexField::from_polar(0.0, PI / 3.0);
    assert!(z.re.abs() < EPSILON);
    assert!(z.im.abs() < EPSILON);
}

// =============================================================================
// i() Tests
// =============================================================================

#[test]
fn test_i_unit() {
    let i: Complex<f64> = ComplexField::i();
    assert!((i.re - 0.0).abs() < EPSILON);
    assert!((i.im - 1.0).abs() < EPSILON);
}

#[test]
fn test_i_squared() {
    let i: Complex<f64> = ComplexField::i();
    let i_squared = i * i;
    // i^2 = -1
    assert!((i_squared.re - (-1.0)).abs() < EPSILON);
    assert!(i_squared.im.abs() < EPSILON);
}

// =============================================================================
// is_real() Tests
// =============================================================================

#[test]
fn test_is_real_true() {
    let z = Complex64::new(5.0, 0.0);
    assert!(ComplexField::is_real(&z));
}

#[test]
fn test_is_real_false() {
    let z = Complex64::new(5.0, 1.0);
    assert!(!ComplexField::is_real(&z));
}

#[test]
fn test_is_real_zero() {
    let z = Complex64::new(0.0, 0.0);
    assert!(ComplexField::is_real(&z));
}

#[test]
fn test_is_real_negative() {
    let z = Complex64::new(-5.0, 0.0);
    assert!(ComplexField::is_real(&z));
}

// =============================================================================
// is_imaginary() Tests
// =============================================================================

#[test]
fn test_is_imaginary_true() {
    let z = Complex64::new(0.0, 5.0);
    assert!(ComplexField::is_imaginary(&z));
}

#[test]
fn test_is_imaginary_false() {
    let z = Complex64::new(1.0, 5.0);
    assert!(!ComplexField::is_imaginary(&z));
}

#[test]
fn test_is_imaginary_zero() {
    // Zero is technically both real and imaginary
    let z = Complex64::new(0.0, 0.0);
    assert!(ComplexField::is_imaginary(&z));
}

#[test]
fn test_is_imaginary_negative() {
    let z = Complex64::new(0.0, -5.0);
    assert!(ComplexField::is_imaginary(&z));
}

// =============================================================================
// ComplexField via DivisionAlgebra conjugate/norm_sqr (already in algebra.rs)
// =============================================================================

#[test]
fn test_complex_field_conjugate() {
    let z = Complex64::new(3.0, 4.0);
    let conj = ComplexField::conjugate(&z);
    assert!((conj.re - 3.0).abs() < EPSILON);
    assert!((conj.im - (-4.0)).abs() < EPSILON);
}

#[test]
fn test_complex_field_norm_sqr() {
    let z = Complex64::new(3.0, 4.0);
    let norm_sq = ComplexField::norm_sqr(&z);
    assert!((norm_sq - 25.0).abs() < EPSILON);
}
