/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The synthetic strapdown-IMU model (`ImuModel`): the sensed specific force is the true force plus the
//! accelerometer bias (the drift source), and the accel/gyro-bias and process-noise accessors round-trip
//! the configured spec that seeds the ESKF `Q`.

use deep_causality_cfd::ImuModel;

#[test]
fn sensed_specific_force_adds_the_accel_bias() {
    // The measured specific force is the true force plus the constant accelerometer bias — the error the
    // dead-reckoned nominal accumulates through blackout.
    let imu = ImuModel::<f64>::new([0.1, -0.2, 0.05], [1e-3, 2e-3, -1e-3], [1e-6; 17]);
    let sensed = imu.sense_specific_force([9.81, 0.0, -1.0]);
    assert!((sensed[0] - (9.81 + 0.1)).abs() < 1e-15, "x: {}", sensed[0]);
    assert!((sensed[1] - (-0.2)).abs() < 1e-15, "y: {}", sensed[1]);
    assert!(
        (sensed[2] - (-1.0 + 0.05)).abs() < 1e-15,
        "z: {}",
        sensed[2]
    );
}

#[test]
fn accessors_round_trip_the_configured_spec() {
    // accel_bias / gyro_bias / process_noise return the values the IMU was built with.
    let accel = [0.1f64, -0.2, 0.05];
    let gyro = [1e-3f64, 2e-3, -1e-3];
    let q = [4.2e-5f64; 17];
    let imu = ImuModel::new(accel, gyro, q);
    assert_eq!(imu.accel_bias(), accel, "accel bias must round-trip");
    assert_eq!(imu.gyro_bias(), gyro, "gyro bias must round-trip");
    assert_eq!(imu.process_noise(), q, "process noise must round-trip");
}

#[test]
fn zero_bias_imu_passes_the_true_force_through() {
    // A bias-free IMU senses the true specific force unchanged (the no-op branch of the bias add).
    let imu = ImuModel::<f64>::new([0.0; 3], [0.0; 3], [0.0; 17]);
    let truth = [1.5, -3.0, 9.81];
    assert_eq!(imu.sense_specific_force(truth), truth);
    assert_eq!(imu.accel_bias(), [0.0; 3]);
    assert_eq!(imu.gyro_bias(), [0.0; 3]);
}
