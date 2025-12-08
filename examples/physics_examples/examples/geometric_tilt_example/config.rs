/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the Geometric Tilt Estimator.

/// Process Noise Q - Models uncertainty in gravity estimate (e.g., sensor drift)
/// Set to 0.0 for stationary systems. Typical: 0.0 to 0.001.
pub const Q_DIAG: f64 = 0.0;

/// Base Measurement Noise R - Accelerometer noise variance
/// Lower = trust sensor more. Typical: 0.01 (quality IMU) to 1.0 (noisy).
pub const R_BASE: f64 = 0.1;

/// Motion Detection Threshold - If |accel_magnitude - g| > threshold, skip update
/// To disable: set to f64::MAX. Typical: 0.5 (sensitive) to 2.0 (permissive).
pub const MOTION_THRESHOLD: f64 = 2.0; // m/s²

/// Gyro Scale for Adaptive R - R_effective = R_BASE * (1 + GYRO_SCALE * |gyro|)
/// To disable: set to 0.0. Typical: 0.5 (mild) to 5.0 (aggressive).
pub const GYRO_SCALE: f64 = 2.0;

/// Reference Gravity magnitude
pub const G_REF: f64 = 9.81; // m/s²

/// Tilt Correction Blending Factor - How aggressively to correct toward gravity
/// Lower = smoother but slower. Typical: 0.01 (smooth) to 0.2 (aggressive).
pub const TILT_CORRECTION_ALPHA: f64 = 0.1;
