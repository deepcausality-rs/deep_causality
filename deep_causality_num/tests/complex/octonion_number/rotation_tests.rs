/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Octonion, RealField, Rotation};
use std::f64::consts::{FRAC_PI_2, PI};
use std::fmt::Display;

const EPSILON: f64 = 1e-9;

// Helper function for approximate octonion equality
fn assert_octonion_approx_eq<T: RealField + Display>(
    q1: &Octonion<T>,
    q2: &Octonion<T>,
    epsilon: T,
) {
    assert!(
        (q1.s - q2.s).abs() < epsilon,
        "Scalar part differs: {} vs {}",
        q1.s,
        q2.s
    );
    assert!(
        (q1.e1 - q2.e1).abs() < epsilon,
        "e1 part differs: {} vs {}",
        q1.e1,
        q2.e1
    );
    assert!(
        (q1.e2 - q2.e2).abs() < epsilon,
        "e2 part differs: {} vs {}",
        q1.e2,
        q2.e2
    );
    assert!(
        (q1.e3 - q2.e3).abs() < epsilon,
        "e3 part differs: {} vs {}",
        q1.e3,
        q2.e3
    );
    assert!(
        (q1.e4 - q2.e4).abs() < epsilon,
        "e4 part differs: {} vs {}",
        q1.e4,
        q2.e4
    );
    assert!(
        (q1.e5 - q2.e5).abs() < epsilon,
        "e5 part differs: {} vs {}",
        q1.e5,
        q2.e5
    );
    assert!(
        (q1.e6 - q2.e6).abs() < epsilon,
        "e6 part differs: {} vs {}",
        q1.e6,
        q2.e6
    );
    assert!(
        (q1.e7 - q2.e7).abs() < epsilon,
        "e7 part differs: {} vs {}",
        q1.e7,
        q2.e7
    );
}

// Tests for rotate_x
#[test]
fn test_octonion_rotate_x_e2_90_deg() {
    let oct = Octonion::<f64>::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure e2
    let rotated_oct = oct.rotate_x(FRAC_PI_2); // Rotate 90 degrees around e1 (x-axis)

    let expected_oct = Octonion::<f64>::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0); // Expected pure e3
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

#[test]
fn test_octonion_rotate_x_e2_180_deg() {
    let oct = Octonion::<f64>::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure e2
    let rotated_oct = oct.rotate_x(PI); // Rotate 180 degrees around e1 (x-axis)

    let expected_oct = Octonion::<f64>::new(0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Expected pure -e2
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

#[test]
fn test_octonion_rotate_x_e2_360_deg() {
    let oct = Octonion::<f64>::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure e2
    let rotated_oct = oct.rotate_x(2.0 * PI); // Rotate 360 degrees around e1 (x-axis)

    let expected_oct = Octonion::<f64>::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Expected pure e2 (back to original)
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

#[test]
fn test_octonion_rotate_x_scalar_part() {
    let oct = Octonion::<f64>::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure scalar
    let rotated_oct = oct.rotate_x(FRAC_PI_2); // Rotate 90 degrees around e1 (x-axis)

    let expected_oct = Octonion::<f64>::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Expected to be unchanged
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

#[test]
fn test_octonion_rotate_x_e1_axis() {
    let oct = Octonion::<f64>::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure e1
    let rotated_oct = oct.rotate_x(FRAC_PI_2); // Rotate 90 degrees around e1 (x-axis)

    let expected_oct = Octonion::<f64>::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Expected to be unchanged
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

// Tests for rotate_y
#[test]
fn test_octonion_rotate_y_e1_90_deg() {
    let oct = Octonion::<f64>::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure e1
    let rotated_oct = oct.rotate_y(FRAC_PI_2); // Rotate 90 degrees around e2 (y-axis)

    let expected_oct = Octonion::<f64>::new(0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0); // Expected pure -e3
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

#[test]
fn test_octonion_rotate_y_e1_180_deg() {
    let oct = Octonion::<f64>::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure e1
    let rotated_oct = oct.rotate_y(PI); // Rotate 180 degrees around e2 (y-axis)

    let expected_oct = Octonion::<f64>::new(0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Expected pure -e1
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

#[test]
fn test_octonion_rotate_y_e2_axis() {
    let oct = Octonion::<f64>::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure e2
    let rotated_oct = oct.rotate_y(FRAC_PI_2); // Rotate 90 degrees around e2 (y-axis)

    let expected_oct = Octonion::<f64>::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Expected to be unchanged
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

// Tests for rotate_z
#[test]
fn test_octonion_rotate_z_e1_90_deg() {
    let oct = Octonion::<f64>::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure e1
    let rotated_oct = oct.rotate_z(FRAC_PI_2); // Rotate 90 degrees around e3 (z-axis)

    let expected_oct = Octonion::<f64>::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Expected pure e2
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

#[test]
fn test_octonion_rotate_z_e1_180_deg() {
    let oct = Octonion::<f64>::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // pure e1
    let rotated_oct = oct.rotate_z(PI); // Rotate 180 degrees around e3 (z-axis)

    let expected_oct = Octonion::<f64>::new(0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Expected pure -e1
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

#[test]
fn test_octonion_rotate_z_e3_axis() {
    let oct = Octonion::<f64>::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0); // pure e3
    let rotated_oct = oct.rotate_z(FRAC_PI_2); // Rotate 90 degrees around e3 (z-axis)

    let expected_oct = Octonion::<f64>::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0); // Expected to be unchanged
    assert_octonion_approx_eq(&rotated_oct, &expected_oct, EPSILON);
}

// Tests for global_phase
#[test]
fn test_octonion_global_phase_returns_self() {
    let oct = Octonion::<f64>::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let angle = PI / 3.0; // Arbitrary angle

    let result_oct = oct.global_phase(angle);

    // Expect the original octonion back
    assert_octonion_approx_eq(&result_oct, &oct, EPSILON);
}

#[test]
fn test_octonion_global_phase_identity() {
    let oct_identity = Octonion::<f64>::identity();
    let angle = PI / 4.0;

    let result_oct = oct_identity.global_phase(angle);

    // Expect the original identity octonion back
    assert_octonion_approx_eq(&result_oct, &oct_identity, EPSILON);
}
