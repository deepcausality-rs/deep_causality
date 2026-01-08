/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, RealField, Rotation, ToPrimitive};
use std::f64::consts::{FRAC_PI_2, PI};

// Helper for approximate floating point comparison
const EPSILON: f64 = 1e-9;

fn assert_complex_approx_eq<T: RealField + ToPrimitive>(a: Complex<T>, b: Complex<T>) {
    let a_re = a.re.to_f64().unwrap();
    let a_im = a.im.to_f64().unwrap();
    let b_re = b.re.to_f64().unwrap();
    let b_im = b.im.to_f64().unwrap();

    assert!(
        (a_re - b_re).abs() < EPSILON,
        "Real parts differ: {} vs {}",
        a_re,
        b_re
    );
    assert!(
        (a_im - b_im).abs() < EPSILON,
        "Imaginary parts differ: {} vs {}",
        a_im,
        b_im
    );
}

#[test]
fn test_rotate_x_no_change() {
    let c = Complex::new(1.0f64, 2.0f64);
    let rotated_c = c.rotate_x(PI / 2.0);
    assert_complex_approx_eq(c, rotated_c);
}

#[test]
fn test_rotate_y_no_change() {
    let c = Complex::new(1.0f64, 2.0f64);
    let rotated_c = c.rotate_y(PI / 2.0);
    assert_complex_approx_eq(c, rotated_c);
}

#[test]
fn test_global_phase_rotate_0() {
    let c = Complex::new(1.0f64, 0.0f64); // (1, 0)
    let rotated_c = c.global_phase(0.0f64);
    assert_complex_approx_eq(Complex::new(1.0f64, 0.0f64), rotated_c);
}

#[test]
fn test_global_phase_rotate_pi_2() {
    let c = Complex::new(1.0f64, 0.0f64); // (1, 0)
    let rotated_c = c.global_phase(FRAC_PI_2); // Rotate by 90 degrees (pi/2)
    assert_complex_approx_eq(Complex::new(0.0f64, 1.0f64), rotated_c); // Should be (0, 1)
}

#[test]
fn test_global_phase_rotate_pi() {
    let c = Complex::new(1.0f64, 0.0f64); // (1, 0)
    let rotated_c = c.global_phase(PI); // Rotate by 180 degrees (pi)
    assert_complex_approx_eq(Complex::new(-1.0f64, 0.0f64), rotated_c); // Should be (-1, 0)
}

#[test]
fn test_global_phase_rotate_3pi_2() {
    let c = Complex::new(1.0f64, 0.0f64); // (1, 0)
    let rotated_c = c.global_phase(3.0 * FRAC_PI_2); // Rotate by 270 degrees (3pi/2)
    assert_complex_approx_eq(Complex::new(0.0f64, -1.0f64), rotated_c); // Should be (0, -1)
}

#[test]
fn test_global_phase_rotate_2pi() {
    let c = Complex::new(1.0f64, 0.0f64); // (1, 0)
    let rotated_c = c.global_phase(2.0 * PI); // Rotate by 360 degrees (2pi)
    assert_complex_approx_eq(Complex::new(1.0f64, 0.0f64), rotated_c); // Should be (1, 0)
}

#[test]
fn test_global_phase_rotate_negative_angle() {
    let c = Complex::new(1.0f64, 0.0f64); // (1, 0)
    let rotated_c = c.global_phase(-FRAC_PI_2); // Rotate by -90 degrees (-pi/2)
    assert_complex_approx_eq(Complex::new(0.0f64, -1.0f64), rotated_c); // Should be (0, -1)
}

#[test]
fn test_global_phase_rotate_arbitrary_complex() {
    let c = Complex::new(3.0f64, 4.0f64); // (3, 4)
    let angle = FRAC_PI_2; // Rotate by 90 degrees
    // Expected result: (3 + 4i) * (cos(pi/2) + i*sin(pi/2)) = (3 + 4i) * (0 + 1i) = -4 + 3i
    let expected = Complex::new(-4.0f64, 3.0f64);
    let rotated_c = c.global_phase(angle);
    assert_complex_approx_eq(expected, rotated_c);
}

#[test]
fn test_rotate_z() {
    let c = Complex::new(1.0f64, 0.0f64); // (1, 0)
    let rotated_c = c.rotate_z(FRAC_PI_2); // Rotate by 90 degrees (pi/2)
    assert_complex_approx_eq(Complex::new(0.0f64, 1.0f64), rotated_c); // Should be (0, 1)
}

#[test]
fn test_rotate_z_multi_rotation() {
    let c = Complex::new(1.0f64, 0.0f64);
    let r1 = c.rotate_z(FRAC_PI_2); // 0 + 1i
    let r2 = r1.rotate_z(FRAC_PI_2); // -1 + 0i
    let r3 = r2.rotate_z(FRAC_PI_2); // 0 - 1i
    let r4 = r3.rotate_z(FRAC_PI_2); // 1 + 0i

    assert_complex_approx_eq(Complex::new(0.0f64, 1.0f64), r1);
    assert_complex_approx_eq(Complex::new(-1.0f64, 0.0f64), r2);
    assert_complex_approx_eq(Complex::new(0.0f64, -1.0f64), r3);
    assert_complex_approx_eq(Complex::new(1.0f64, 0.0f64), r4);
}
