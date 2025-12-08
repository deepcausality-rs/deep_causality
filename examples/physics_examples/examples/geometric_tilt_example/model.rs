/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model types and causaloid functions for the Geometric Tilt Estimator.

use crate::config::*;
use deep_causality::PropagatingEffect;
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector, MultiVectorL2Norm};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

// ============================================================================
// Data Structures
// ============================================================================

/// Input sensor data for a single time step
#[derive(Clone, Debug, Default)]
pub struct SensorData {
    pub accel: [f64; 3], // Accelerometer [ax, ay, az] m/s²
    pub gyro: [f64; 3],  // Gyroscope [wx, wy, wz] rad/s
    pub dt: f64,         // Time step in seconds
}

/// State of the Tilt Estimator
#[derive(Clone, Debug, Default)]
pub struct TiltState {
    /// Current orientation estimate (Rotor in Geometric Algebra)
    pub orientation: Option<CausalMultiVector<f64>>,

    /// Estimated gravity vector in Body Frame
    pub gravity_body: Option<CausalMultiVector<f64>>,

    /// Covariance matrix P for Kalman filter [3x3]
    pub covariance: Option<CausalTensor<f64>>,

    /// Motion detection flag from last update
    pub motion_detected: bool,

    /// Gyro magnitude from last update
    pub gyro_magnitude: f64,
}

impl TiltState {
    /// Get trace of covariance matrix (sum of diagonal)
    pub fn covariance_trace(&self) -> f64 {
        self.covariance
            .as_ref()
            .map(|p| {
                let d = p.as_slice();
                d.first().unwrap_or(&0.0) + d.get(4).unwrap_or(&0.0) + d.get(8).unwrap_or(&0.0)
            })
            .unwrap_or(0.0)
    }
}

// ============================================================================
// Initialization
// ============================================================================

/// Create initial state with identity orientation and default gravity
pub fn create_initial_state() -> Result<TiltState, Box<dyn std::error::Error>> {
    let metric = Metric::Euclidean(3);

    // Identity rotor (scalar = 1.0)
    let mut orientation_data = vec![0.0; 8];
    orientation_data[0] = 1.0;
    let orientation = CausalMultiVector::new(orientation_data, metric)?;

    // Initial gravity estimate: [0, 0, 9.81] in body frame (NED convention)
    let gravity_body = create_vector(&[0.0, 0.0, 9.81], &metric)?;

    // Initial covariance: 100 * I (high uncertainty)
    let covariance = CausalTensor::new(
        vec![100.0, 0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 100.0],
        vec![3, 3],
    )?;

    Ok(TiltState {
        orientation: Some(orientation),
        gravity_body: Some(gravity_body),
        covariance: Some(covariance),
        motion_detected: false,
        gyro_magnitude: 0.0,
    })
}

// ============================================================================
// Causaloid Functions (Steps in the Causal Chain)
// ============================================================================

/// Step 1: Integrate gyroscope to predict orientation
///
/// Uses first-order rotor update: R_new = R_old * (1 - 0.5 * Ω * dt)
pub fn integrate_gyro(state: TiltState, sensor: &SensorData) -> PropagatingEffect<TiltState> {
    let metric = Metric::Euclidean(3);
    let dt = sensor.dt;

    // Create bivector from gyro angular velocity
    let omega = match create_bivector_from_gyro(&sensor.gyro, &metric) {
        Ok(bv) => bv,
        Err(_) => return PropagatingEffect::pure(state),
    };

    // Rotor update: exp(-0.5 * Ω * dt) ≈ 1 - 0.5 * Ω * dt (first order)
    let half_omega_dt = omega * (-0.5 * dt);

    let mut one_data = vec![0.0; 8];
    one_data[0] = 1.0;
    let one = match CausalMultiVector::new(one_data, metric) {
        Ok(mv) => mv,
        Err(_) => return PropagatingEffect::pure(state),
    };

    let rotor_update = one + half_omega_dt;

    // Apply to current orientation
    let current_orientation = match state.orientation.clone() {
        Some(o) => o,
        None => return PropagatingEffect::pure(state),
    };

    let predicted_orientation = (current_orientation * rotor_update).normalize_l2();

    // Calculate gyro magnitude for adaptive R
    let gyro_magnitude =
        (sensor.gyro[0].powi(2) + sensor.gyro[1].powi(2) + sensor.gyro[2].powi(2)).sqrt();

    PropagatingEffect::pure(TiltState {
        orientation: Some(predicted_orientation),
        gyro_magnitude,
        ..state
    })
}

/// Step 2: Detect linear acceleration (motion)
///
/// If |accel| significantly differs from g, assume external acceleration is present.
pub fn detect_motion(state: TiltState, sensor: &SensorData) -> PropagatingEffect<TiltState> {
    let accel_magnitude =
        (sensor.accel[0].powi(2) + sensor.accel[1].powi(2) + sensor.accel[2].powi(2)).sqrt();
    let motion_detected = (accel_magnitude - G_REF).abs() > MOTION_THRESHOLD;

    PropagatingEffect::pure(TiltState {
        motion_detected,
        ..state
    })
}

/// Step 3: Kalman Filter update for gravity estimation
///
/// Adaptive Gravity Observer that distinguishes tilting from linear acceleration.
pub fn kalman_update(state: TiltState, sensor: &SensorData) -> PropagatingEffect<TiltState> {
    let (covariance, gravity_body) = match (&state.covariance, &state.gravity_body) {
        (Some(p), Some(g)) => (p.clone(), g.clone()),
        _ => return PropagatingEffect::pure(state),
    };

    // Measurement: accelerometer reading
    let z = match CausalTensor::new(sensor.accel.to_vec(), vec![3, 1]) {
        Ok(t) => t,
        Err(_) => return PropagatingEffect::pure(state),
    };

    // Prediction: current gravity estimate
    let x_pred = CausalTensor::new(
        vec![
            gravity_body.get(1).cloned().unwrap_or(0.0),
            gravity_body.get(2).cloned().unwrap_or(0.0),
            gravity_body.get(3).cloned().unwrap_or(0.0),
        ],
        vec![3, 1],
    )
    .unwrap();

    // Process noise Q
    let q = CausalTensor::new(
        vec![Q_DIAG, 0.0, 0.0, 0.0, Q_DIAG, 0.0, 0.0, 0.0, Q_DIAG],
        vec![3, 3],
    )
    .unwrap();

    // If motion detected, skip measurement update
    if state.motion_detected {
        let p_propagated = &covariance + &q;
        return PropagatingEffect::pure(TiltState {
            covariance: Some(p_propagated),
            ..state
        });
    }

    // Adaptive measurement noise R based on gyro magnitude
    let r_effective = R_BASE * (1.0 + GYRO_SCALE * state.gyro_magnitude);
    let r = CausalTensor::new(
        vec![
            r_effective,
            0.0,
            0.0,
            0.0,
            r_effective,
            0.0,
            0.0,
            0.0,
            r_effective,
        ],
        vec![3, 3],
    )
    .unwrap();

    // Innovation: y = z - x_pred
    let y = &z - &x_pred;

    // Innovation covariance: S = P + R
    let s = &covariance + &r;

    // Kalman gain: K = P * S^-1
    let s_inv = match s.inverse() {
        Ok(inv) => inv,
        Err(_) => return PropagatingEffect::pure(state),
    };
    let k = CausalTensor::ein_sum(&EinSumOp::mat_mul(covariance.clone(), s_inv)).unwrap();

    // State update: x_new = x_pred + K * y
    let correction = CausalTensor::ein_sum(&EinSumOp::mat_mul(k.clone(), y)).unwrap();
    let x_updated = &x_pred + &correction;

    // Covariance update: P_new = (I - K) * P + Q
    let identity = CausalTensor::new(
        vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        vec![3, 3],
    )
    .unwrap();
    let i_minus_k = &identity - &k;
    let p_updated = CausalTensor::ein_sum(&EinSumOp::mat_mul(i_minus_k, covariance)).unwrap();
    let p_with_q = &p_updated + &q;

    // Convert back to MultiVector
    let metric = Metric::Euclidean(3);
    let g_new = create_vector(x_updated.as_slice(), &metric).unwrap();

    PropagatingEffect::pure(TiltState {
        gravity_body: Some(g_new),
        covariance: Some(p_with_q),
        ..state
    })
}

/// Step 4: Apply tilt correction using Geometric Algebra
///
/// Aligns estimated body gravity with reference world gravity using rotors.
pub fn apply_tilt_correction(state: TiltState) -> PropagatingEffect<TiltState> {
    let metric = Metric::Euclidean(3);

    let (orientation, gravity_body) = match (&state.orientation, &state.gravity_body) {
        (Some(o), Some(g)) => (o.clone(), g.clone()),
        _ => return PropagatingEffect::pure(state),
    };

    // Reference gravity in World Frame (NED: down is +Z, but accel reads -Z)
    let g_ref = match create_vector(&[0.0, 0.0, -9.81], &metric) {
        Ok(v) => v,
        Err(_) => return PropagatingEffect::pure(state),
    };

    // Normalize vectors for rotor calculation
    let g_body_norm = gravity_body.normalize();
    let g_ref_norm = g_ref.normalize();

    // Transform g_body to World Frame: g_world = R * g_body * ~R
    let r_rev = orientation.reversion();
    let temp = orientation.geometric_product(&g_body_norm);
    let g_world_est = temp.geometric_product(&r_rev).grade_projection(1);

    // Correction rotor: aligns g_world_est with g_ref
    let one = CausalMultiVector::scalar(1.0, metric);
    let alignment_product = g_ref_norm.geometric_product(&g_world_est);
    let rotor_unnorm = one + alignment_product;
    let correction_rotor = rotor_unnorm.normalize();

    // Blend correction with identity for smooth update
    let identity_rotor = CausalMultiVector::scalar(1.0, metric);
    let weighted_identity = identity_rotor * (1.0 - TILT_CORRECTION_ALPHA);
    let weighted_correction = correction_rotor * TILT_CORRECTION_ALPHA;
    let blended_rotor = (weighted_identity + weighted_correction).normalize();

    // Apply blended correction
    let corrected_orientation = blended_rotor.geometric_product(&orientation).normalize();

    PropagatingEffect::pure(TiltState {
        orientation: Some(corrected_orientation),
        ..state
    })
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a 3D vector multivector
fn create_vector(
    components: &[f64],
    metric: &Metric,
) -> Result<CausalMultiVector<f64>, Box<dyn std::error::Error>> {
    let mut data = vec![0.0; 8];
    if components.len() < 3 {
        return Err("Vector must have 3 components".into());
    }
    data[1] = components[0];
    data[2] = components[1];
    data[3] = components[2];
    Ok(CausalMultiVector::new(data, *metric)?)
}

/// Create a bivector from angular velocity (gyro)
fn create_bivector_from_gyro(
    gyro: &[f64; 3],
    metric: &Metric,
) -> Result<CausalMultiVector<f64>, Box<dyn std::error::Error>> {
    // Bivector Omega = wx*e23 + wy*e31 + wz*e12
    let mut data = vec![0.0; 8];
    data[6] = gyro[0]; // e23
    data[5] = -gyro[1]; // e13 (e31 = -e13)
    data[4] = gyro[2]; // e12
    Ok(CausalMultiVector::new(data, *metric)?)
}
