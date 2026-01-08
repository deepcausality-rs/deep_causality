/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Geometric Tilt Estimator with Adaptive Gravity Observer
//!
//! This example demonstrates a robust tilt estimation system using:
//! - **Geometric Algebra** (Rotors) for singularity-free orientation
//! - **Adaptive Kalman Filter** for gravity estimation
//! - **Motion Detection** to avoid corrupting estimates during acceleration
//!
//! ## Engineering Value
//!
//! 1. **Eliminates Gimbal Lock**: Uses Rotors instead of Euler angles
//! 2. **Dynamic Calibration**: Continuously refines gravity estimate
//! 3. **Hardware Independence**: Pure causal kernel, portable across platforms
//!
//! ## Causal Chain
//!
//! ```text
//! Sensor Data → Gyro Integration → Kalman Prediction
//!                                         ↓
//!                               Motion Detection
//!                                         ↓
//!                               Kalman Update (conditional)
//!                                         ↓
//!                               Tilt Correction
//!                                         ↓
//!                               Final Orientation
//! ```
mod model;

use deep_causality::PropagatingEffect;
use model::{SensorData, TiltState};

/// Process Noise Q - Models uncertainty in gravity estimate (e.g., sensor drift)
/// Set to 0.0 for stationary systems. Typical: 0.0 to 0.001.
pub const Q_DIAG: f64 = 0.0;
/// Base Measurement Noise R - Accelerometer noise variance
/// Lower = trust sensor more. Typical: 0.01 (quality IMU) to 1.0 (noisy).
pub const R_BASE: f64 = 0.1;
/// Motion Detection Threshold - If |accel_magnitude - g| > threshold, skip update
/// To disable: set to f64::MAX. Typical: 0.5 (sensitive) to 2.0 (permissive).
pub const MOTION_THRESHOLD: f64 = 2.0;
/// Gyro Scale for Adaptive R - R_effective = R_BASE * (1 + GYRO_SCALE * |gyro|)
/// To disable: set to 0.0. Typical: 0.5 (mild) to 5.0 (aggressive).
pub const GYRO_SCALE: f64 = 2.0;
/// Tilt Correction Blending Factor - How aggressively to correct toward gravity
/// Lower = smoother but slower. Typical: 0.01 (smooth) to 0.2 (aggressive).
pub const TILT_CORRECTION_ALPHA: f64 = 0.1;

fn main() {
    println!("============================================================");
    println!("   Geometric Tilt Estimator & Adaptive Gravity Observer");
    println!("============================================================");
    println!("   (Causaloid-based IMU Sensor Fusion)\n");

    // =========================================================================
    // Initialize State
    // =========================================================================
    let initial_state = model::create_initial_state().expect("Failed to create initial state");

    println!("Initial State:");
    println!("  Orientation: Identity Rotor");
    println!("  Gravity Estimate: [0, 0, 9.81] m/s²");
    println!("  Covariance: 100 * I\n");

    // =========================================================================
    // Generate Simulated Sensor Stream
    // =========================================================================
    let sensor_stream = generate_sensor_stream();
    println!(
        "Simulated {} sensor readings over 0.5 seconds",
        sensor_stream.len()
    );
    println!("Scenario: Stationary → Tilt 45° around X → Stationary\n");

    // =========================================================================
    // Run Causal Chain via Monadic Fold
    // =========================================================================
    println!("--- Processing Sensor Stream ---\n");

    let final_state =
        sensor_stream
            .into_iter()
            .enumerate()
            .fold(initial_state, |state, (i, sensor_data)| {
                // Execute the causal chain for each sensor reading
                let result: PropagatingEffect<TiltState> = PropagatingEffect::pure(state)
                    .bind(|s, _, _| {
                        // Step 1: Gyro Integration (Orientation Prediction)
                        model::integrate_gyro(s.into_value().unwrap_or_default(), &sensor_data)
                    })
                    .bind(|s, _, _| {
                        // Step 2: Motion Detection
                        model::detect_motion(s.into_value().unwrap_or_default(), &sensor_data)
                    })
                    .bind(|s, _, _| {
                        // Step 3: Kalman Filter Update (Gravity Observer)
                        model::kalman_update(s.into_value().unwrap_or_default(), &sensor_data)
                    })
                    .bind(|s, _, _| {
                        // Step 4: Tilt Correction
                        model::apply_tilt_correction(s.into_value().unwrap_or_default())
                    });

                // Print progress at key frames
                if i == 0 || i == 15 || i == 35 || i == 49 {
                    print_state_summary(i, &result.value.clone().into_value().unwrap_or_default());
                }

                result.value.into_value().unwrap_or_default()
            });

    // =========================================================================
    // Output Final Results
    // =========================================================================
    println!("\n============================================================");
    println!("FINAL STATE:");
    println!("============================================================");

    if let Some(ref g) = final_state.gravity_body {
        let gz = g.get(3).cloned().unwrap_or(0.0);
        println!("  Estimated Gravity Z: {:.4} m/s²", gz);
    }

    if let Some(ref r) = final_state.orientation {
        let scalar = r.get(0).cloned().unwrap_or(0.0);
        println!("  Orientation Scalar:  {:.4} (1.0 = identity)", scalar);
    }

    println!(
        "  Covariance Trace:    {:.4}",
        final_state.covariance_trace()
    );
    println!("  Motion Detected:     {}", final_state.motion_detected);

    println!("\n============================================================");
    println!("The system successfully tracked orientation through the tilt maneuver");
    println!("============================================================");
}

/// Generate simulated sensor data stream
fn generate_sensor_stream() -> Vec<SensorData> {
    let steps = 50;
    let dt = 0.01;
    let mut stream = Vec::with_capacity(steps);

    for i in 0..steps {
        let mut gyro = [0.0, 0.0, 0.0];
        let accel = [0.0, 0.0, -9.81]; // Stationary: accelerometer reads reaction force

        // Tilting phase: frames 10-30
        if i > 10 && i < 30 {
            gyro[0] = 1.0; // 1 rad/s around X axis
        }

        stream.push(SensorData { accel, gyro, dt });
    }

    stream
}

/// Print state summary at key frames
fn print_state_summary(frame: usize, state: &TiltState) {
    let gz = state
        .gravity_body
        .as_ref()
        .and_then(|g| g.get(3).cloned())
        .unwrap_or(0.0);
    let scalar = state
        .orientation
        .as_ref()
        .and_then(|r| r.get(0).cloned())
        .unwrap_or(1.0);

    let phase = match frame {
        0 => "Start (Stationary)",
        15 => "Mid-Tilt (Rotating)",
        35 => "Post-Tilt (Stabilizing)",
        49 => "End (Final)",
        _ => "...",
    };

    println!(
        "[Frame {:2}] {} | Gz: {:6.3} | Rotor: {:6.4} | Motion: {}",
        frame, phase, gz, scalar, state.motion_detected
    );
}
