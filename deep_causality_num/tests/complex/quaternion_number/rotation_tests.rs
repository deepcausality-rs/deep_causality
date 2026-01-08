/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::RealField;
use deep_causality_num::{Quaternion, Rotation};
use std::f64::consts::{FRAC_PI_2, PI};

const EPSILON: f64 = 1e-9;

#[test]
fn test_to_axis_angle_identity() {
    // Identity quaternion
    let q_identity = Quaternion::<f64>::identity();
    let (axis, angle) = q_identity.to_axis_angle();
    assert!((angle - 0.0).abs() < EPSILON);
    // For identity, axis can be arbitrary, so we check if it's a unit vector
    assert!((axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2] - 1.0).abs() < EPSILON);
}

#[test]
fn test_to_axis_angle_x_90() {
    // 90 degrees around X axis
    let q_x_90 = Quaternion::from_axis_angle([1.0, 0.0, 0.0], std::f64::consts::FRAC_PI_2);
    let (axis, angle) = q_x_90.to_axis_angle();
    assert!((axis[0] - 1.0).abs() < EPSILON);
    assert!((axis[1] - 0.0).abs() < EPSILON);
    assert!((axis[2] - 0.0).abs() < EPSILON);
    assert!((angle - std::f64::consts::FRAC_PI_2).abs() < EPSILON);
}

#[test]
fn test_to_axis_angle_y_180() {
    // 180 degrees around Y axis
    let q_y_180 = Quaternion::from_axis_angle([0.0, 1.0, 0.0], std::f64::consts::PI);
    let (axis, angle) = q_y_180.to_axis_angle();
    assert!((angle - std::f64::consts::PI).abs() < EPSILON);
    // Check if axis is parallel to [0, 1, 0]
    let expected_axis = [0.0, 1.0, 0.0];
    let dot_product =
        axis[0] * expected_axis[0] + axis[1] * expected_axis[1] + axis[2] * expected_axis[2];
    assert!((dot_product.abs() - 1.0).abs() < EPSILON); // Check if dot product is 1 or -1
}

#[test]
fn test_to_axis_angle_z_360() {
    // 360 degrees around Z axis (equivalent to 0 degrees)
    let q_z_360 = Quaternion::from_axis_angle([0.0, 0.0, 1.0], std::f64::consts::TAU);
    let (axis, angle) = q_z_360.to_axis_angle();
    assert!((angle - 0.0).abs() < EPSILON);
    // For 0 angle, axis can be arbitrary, so we check if it's a unit vector
    assert!((axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2] - 1.0).abs() < EPSILON);
}

#[test]
fn test_to_rotation_matrix_identity() {
    // Identity quaternion
    let q_identity = Quaternion::<f64>::identity();
    let mat_identity = q_identity.to_rotation_matrix();
    assert!((mat_identity[0][0] - 1.0).abs() < EPSILON);
    assert!((mat_identity[0][1] - 0.0).abs() < EPSILON);
    assert!((mat_identity[0][2] - 0.0).abs() < EPSILON);
    assert!((mat_identity[1][0] - 0.0).abs() < EPSILON);
    assert!((mat_identity[1][1] - 1.0).abs() < EPSILON);
    assert!((mat_identity[1][2] - 0.0).abs() < EPSILON);
    assert!((mat_identity[2][0] - 0.0).abs() < EPSILON);
    assert!((mat_identity[2][1] - 0.0).abs() < EPSILON);
    assert!((mat_identity[2][2] - 1.0).abs() < EPSILON);
}

#[test]
fn test_to_rotation_matrix_x_90() {
    // 90 degrees around X axis
    let q_x_90 = Quaternion::from_axis_angle([1.0, 0.0, 0.0], std::f64::consts::FRAC_PI_2);
    let mat_x_90 = q_x_90.to_rotation_matrix();
    assert!((mat_x_90[0][0] - 1.0).abs() < EPSILON);
    assert!((mat_x_90[0][1] - 0.0).abs() < EPSILON);
    assert!((mat_x_90[0][2] - 0.0).abs() < EPSILON);
    assert!((mat_x_90[1][0] - 0.0).abs() < EPSILON);
    assert!((mat_x_90[1][1] - 0.0).abs() < EPSILON);
    assert!((mat_x_90[1][2] - (-1.0)).abs() < EPSILON);
    assert!((mat_x_90[2][0] - 0.0).abs() < EPSILON);
    assert!((mat_x_90[2][1] - 1.0).abs() < EPSILON);
    assert!((mat_x_90[2][2] - 0.0).abs() < EPSILON);
}

#[test]
fn test_to_rotation_matrix_y_180() {
    // 180 degrees around Y axis
    let q_y_180 = Quaternion::from_axis_angle([0.0, 1.0, 0.0], std::f64::consts::PI);
    let mat_y_180 = q_y_180.to_rotation_matrix();
    assert!((mat_y_180[0][0] - (-1.0)).abs() < EPSILON);
    assert!((mat_y_180[0][1] - 0.0).abs() < EPSILON);
    assert!((mat_y_180[0][2] - 0.0).abs() < EPSILON);
    assert!((mat_y_180[1][0] - 0.0).abs() < EPSILON);
    assert!((mat_y_180[1][1] - 1.0).abs() < EPSILON);
    assert!((mat_y_180[1][2] - 0.0).abs() < EPSILON);
    assert!((mat_y_180[2][0] - 0.0).abs() < EPSILON);
    assert!((mat_y_180[2][1] - 0.0).abs() < EPSILON);
    assert!((mat_y_180[2][2] - (-1.0)).abs() < EPSILON);
}

#[test]
fn test_to_rotation_matrix_x_270() {
    // 270 degrees around X axis
    let q_x_270 = Quaternion::from_axis_angle([1.0, 0.0, 0.0], 3.0 * std::f64::consts::FRAC_PI_2);
    let mat_x_270 = q_x_270.to_rotation_matrix();
    assert!((mat_x_270[0][0] - 1.0).abs() < EPSILON);
    assert!((mat_x_270[0][1] - 0.0).abs() < EPSILON);
    assert!((mat_x_270[0][2] - 0.0).abs() < EPSILON);
    assert!((mat_x_270[1][0] - 0.0).abs() < EPSILON);
    assert!((mat_x_270[1][1] - 0.0).abs() < EPSILON);
    assert!((mat_x_270[1][2] - 1.0).abs() < EPSILON);
    assert!((mat_x_270[2][0] - 0.0).abs() < EPSILON);
    assert!((mat_x_270[2][1] - (-1.0)).abs() < EPSILON);
    assert!((mat_x_270[2][2] - 0.0).abs() < EPSILON);
}

#[test]
fn test_to_rotation_matrix_z_360() {
    // 360 degrees around Z axis (equivalent to identity)
    let q_z_360 = Quaternion::from_axis_angle([0.0, 0.0, 1.0], std::f64::consts::TAU);
    let mat_z_360 = q_z_360.to_rotation_matrix();
    assert!((mat_z_360[0][0] - 1.0).abs() < EPSILON);
    assert!((mat_z_360[0][1] - 0.0).abs() < EPSILON);
    assert!((mat_z_360[0][2] - 0.0).abs() < EPSILON);
    assert!((mat_z_360[1][0] - 0.0).abs() < EPSILON);
    assert!((mat_z_360[1][1] - 1.0).abs() < EPSILON);
    assert!((mat_z_360[1][2] - 0.0).abs() < EPSILON);
    assert!((mat_z_360[2][0] - 0.0).abs() < EPSILON);
    assert!((mat_z_360[2][1] - 0.0).abs() < EPSILON);
    assert!((mat_z_360[2][2] - 1.0).abs() < EPSILON);
}

#[test]
fn test_slerp_t_0() {
    let q1 = Quaternion::<f64>::identity();
    let q2 = Quaternion::from_axis_angle([1.0, 0.0, 0.0], std::f64::consts::FRAC_PI_2);

    // t = 0, should be q1
    let slerp_0 = q1.slerp(&q2, 0.0);
    assert!((slerp_0.w - q1.w).abs() < EPSILON);
    assert!((slerp_0.x - q1.x).abs() < EPSILON);
    assert!((slerp_0.y - q1.y).abs() < EPSILON);
    assert!((slerp_0.z - q1.z).abs() < EPSILON);
}

#[test]
fn test_slerp_t_1() {
    let q1 = Quaternion::<f64>::identity();
    let q2 = Quaternion::from_axis_angle([1.0, 0.0, 0.0], std::f64::consts::FRAC_PI_2);

    // t = 1, should be q2
    let slerp_1 = q1.slerp(&q2, 1.0);
    assert!((slerp_1.w - q2.w).abs() < EPSILON);
    assert!((slerp_1.x - q2.x).abs() < EPSILON);
    assert!((slerp_1.y - q2.y).abs() < EPSILON);
    assert!((slerp_1.z - q2.z).abs() < EPSILON);
}

#[test]
fn test_slerp_t_0_5() {
    let q1 = Quaternion::<f64>::identity();
    let q2 = Quaternion::from_axis_angle([1.0, 0.0, 0.0], std::f64::consts::FRAC_PI_2);

    // t = 0.5, should be half-way rotation
    let slerp_0_5 = q1.slerp(&q2, 0.5);
    let expected_half = Quaternion::from_axis_angle([1.0, 0.0, 0.0], std::f64::consts::FRAC_PI_4);
    assert!((slerp_0_5.w - expected_half.w).abs() < EPSILON);
    assert!((slerp_0_5.x - expected_half.x).abs() < EPSILON);
    assert!((slerp_0_5.y - expected_half.y).abs() < EPSILON);
    assert!((slerp_0_5.z - expected_half.z).abs() < EPSILON);
}

#[test]
fn test_slerp_antipodal() {
    // Test with opposite quaternions (dot product < 0)
    let q3 = Quaternion::new(0.0, 0.0, 0.0, 1.0); // 180 deg around Z
    let q4 = Quaternion::new(0.0, 0.0, 0.0, -1.0); // -180 deg around Z
    let slerp_opposite = q3.slerp(&q4, 0.5);

    // Should be q3 (0,0,0,1) after negation and linear interpolation
    assert!((slerp_opposite.w - 0.0).abs() < EPSILON);
    assert!((slerp_opposite.x - 0.0).abs() < EPSILON);
    assert!((slerp_opposite.y - 0.0).abs() < EPSILON);
    assert!((slerp_opposite.z - 1.0).abs() < EPSILON);
}

#[test]
fn test_slerp_nearly_identical() {
    let q1 = Quaternion::<f64>::identity();
    // Test with nearly identical quaternions
    let q5 = Quaternion::new(1.0, 0.000000001, 0.0, 0.0).normalize();
    let slerp_close = q1.slerp(&q5, 0.5);
    assert!((slerp_close.w - q1.w).abs() < EPSILON);
    assert!((slerp_close.x - q1.x).abs() < EPSILON);
    assert!((slerp_close.y - q1.y).abs() < EPSILON);
    assert!((slerp_close.z - q1.z).abs() < EPSILON);
}

// Tests for rotate_x
#[test]
fn test_rotate_x_identity_90() {
    let q_identity = Quaternion::<f64>::identity();
    let rotated_q = q_identity.rotate_x(FRAC_PI_2); // Rotate 90 degrees around X

    // Expected: Quaternion representing 90 deg rotation around X
    let expected_q = Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2);

    assert!((rotated_q.w - expected_q.w).abs() < EPSILON);
    assert!((rotated_q.x - expected_q.x).abs() < EPSILON);
    assert!((rotated_q.y - expected_q.y).abs() < EPSILON);
    assert!((rotated_q.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_rotate_x_identity_180() {
    let q_identity = Quaternion::<f64>::identity();
    let rotated_q = q_identity.rotate_x(PI); // Rotate 180 degrees around X

    // Expected: Quaternion representing 180 deg rotation around X
    let expected_q = Quaternion::from_axis_angle([1.0, 0.0, 0.0], PI);

    assert!((rotated_q.w - expected_q.w).abs() < EPSILON);
    assert!((rotated_q.x - expected_q.x).abs() < EPSILON);
    assert!((rotated_q.y - expected_q.y).abs() < EPSILON);
    assert!((rotated_q.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_rotate_x_initial_y_90_then_x_90() {
    // Initial quaternion: 90 degrees around Y
    let initial_q = Quaternion::from_axis_angle([0.0, 1.0, 0.0], FRAC_PI_2);
    let rotated_q = initial_q.rotate_x(FRAC_PI_2); // Rotate an additional 90 degrees around X

    // Expected: The resulting quaternion from composing these rotations.
    // Order matters: first Y, then X.
    // Equivalent to (Qx * Qy) where Qx is 90 deg around X and Qy is 90 deg around Y.
    let q_x = Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2);
    let expected_q = q_x * initial_q;

    assert!((rotated_q.w - expected_q.w).abs() < EPSILON);
    assert!((rotated_q.x - expected_q.x).abs() < EPSILON);
    assert!((rotated_q.y - expected_q.y).abs() < EPSILON);
    assert!((rotated_q.z - expected_q.z).abs() < EPSILON);
}

// Tests for rotate_y
#[test]
fn test_rotate_y_identity_90() {
    let q_identity = Quaternion::<f64>::identity();
    let rotated_q = q_identity.rotate_y(FRAC_PI_2); // Rotate 90 degrees around Y

    // Expected: Quaternion representing 90 deg rotation around Y
    let expected_q = Quaternion::from_axis_angle([0.0, 1.0, 0.0], FRAC_PI_2);

    assert!((rotated_q.w - expected_q.w).abs() < EPSILON);
    assert!((rotated_q.x - expected_q.x).abs() < EPSILON);
    assert!((rotated_q.y - expected_q.y).abs() < EPSILON);
    assert!((rotated_q.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_rotate_y_identity_180() {
    let q_identity = Quaternion::<f64>::identity();
    let rotated_q = q_identity.rotate_y(PI); // Rotate 180 degrees around Y

    // Expected: Quaternion representing 180 deg rotation around Y
    let expected_q = Quaternion::from_axis_angle([0.0, 1.0, 0.0], PI);

    assert!((rotated_q.w - expected_q.w).abs() < EPSILON);
    assert!((rotated_q.x - expected_q.x).abs() < EPSILON);
    assert!((rotated_q.y - expected_q.y).abs() < EPSILON);
    assert!((rotated_q.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_rotate_y_initial_x_90_then_y_90() {
    // Initial quaternion: 90 degrees around X
    let initial_q = Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2);
    let rotated_q = initial_q.rotate_y(FRAC_PI_2); // Rotate an additional 90 degrees around Y

    // Expected: The resulting quaternion from composing these rotations.
    // Equivalent to (Qy * Qx) where Qy is 90 deg around Y and Qx is 90 deg around X.
    let q_y = Quaternion::from_axis_angle([0.0, 1.0, 0.0], FRAC_PI_2);
    let expected_q = q_y * initial_q;

    assert!((rotated_q.w - expected_q.w).abs() < EPSILON);
    assert!((rotated_q.x - expected_q.x).abs() < EPSILON);
    assert!((rotated_q.y - expected_q.y).abs() < EPSILON);
    assert!((rotated_q.z - expected_q.z).abs() < EPSILON);
}

// Tests for rotate_z
#[test]
fn test_rotate_z_identity_90() {
    let q_identity = Quaternion::<f64>::identity();
    let rotated_q = q_identity.rotate_z(FRAC_PI_2); // Rotate 90 degrees around Z

    // Expected: Quaternion representing 90 deg rotation around Z
    let expected_q = Quaternion::from_axis_angle([0.0, 0.0, 1.0], FRAC_PI_2);

    assert!((rotated_q.w - expected_q.w).abs() < EPSILON);
    assert!((rotated_q.x - expected_q.x).abs() < EPSILON);
    assert!((rotated_q.y - expected_q.y).abs() < EPSILON);
    assert!((rotated_q.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_rotate_z_identity_180() {
    let q_identity = Quaternion::<f64>::identity();
    let rotated_q = q_identity.rotate_z(PI); // Rotate 180 degrees around Z

    // Expected: Quaternion representing 180 deg rotation around Z
    let expected_q = Quaternion::from_axis_angle([0.0, 0.0, 1.0], PI);

    assert!((rotated_q.w - expected_q.w).abs() < EPSILON);
    assert!((rotated_q.x - expected_q.x).abs() < EPSILON);
    assert!((rotated_q.y - expected_q.y).abs() < EPSILON);
    assert!((rotated_q.z - expected_q.z).abs() < EPSILON);
}

#[test]
fn test_rotate_z_initial_x_90_then_z_90() {
    // Initial quaternion: 90 degrees around X
    let initial_q = Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2);
    let rotated_q = initial_q.rotate_z(FRAC_PI_2); // Rotate an additional 90 degrees around Z

    // Expected: The resulting quaternion from composing these rotations.
    // Equivalent to (Qz * Qx) where Qz is 90 deg around Z and Qx is 90 deg around X.
    let q_z = Quaternion::from_axis_angle([0.0, 0.0, 1.0], FRAC_PI_2);
    let expected_q = q_z * initial_q;

    assert!((rotated_q.w - expected_q.w).abs() < EPSILON);
    assert!((rotated_q.x - expected_q.x).abs() < EPSILON);
    assert!((rotated_q.y - expected_q.y).abs() < EPSILON);
    assert!((rotated_q.z - expected_q.z).abs() < EPSILON);
}

// Tests for global_phase
#[test]
fn test_global_phase_returns_self() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let angle = PI / 3.0; // Arbitrary angle

    let result_q = q.global_phase(angle);

    // Expect the original quaternion back
    assert!((result_q.w - q.w).abs() < EPSILON);
    assert!((result_q.x - q.x).abs() < EPSILON);
    assert!((result_q.y - q.y).abs() < EPSILON);
    assert!((result_q.z - q.z).abs() < EPSILON);
}

#[test]
fn test_global_phase_identity() {
    let q_identity = Quaternion::<f64>::identity();
    let angle = PI / 4.0;

    let result_q = q_identity.global_phase(angle);

    // Expect the original identity quaternion back
    assert!((result_q.w - q_identity.w).abs() < EPSILON);
    assert!((result_q.x - q_identity.x).abs() < EPSILON);
    assert!((result_q.y - q_identity.y).abs() < EPSILON);
    assert!((result_q.z - q_identity.z).abs() < EPSILON);
}
