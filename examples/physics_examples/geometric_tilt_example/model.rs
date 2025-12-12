/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model types and causaloid functions for the Geometric Tilt Estimator.

use crate::{GYRO_SCALE, MOTION_THRESHOLD, Q_DIAG, R_BASE, TILT_CORRECTION_ALPHA};
use deep_causality::PropagatingEffect;
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector, MultiVectorL2Norm};
use deep_causality_physics::G;
use deep_causality_physics::kalman_filter_linear;
use deep_causality_tensor::CausalTensor;
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
    let motion_detected = (accel_magnitude - G).abs() > MOTION_THRESHOLD;

    PropagatingEffect::pure(TiltState {
        motion_detected,
        ..state
    })
}

/// Step 3: Kalman Filter update for gravity estimation
///
/// Adaptive Gravity Observer that distinguishes tilting from linear acceleration.
pub fn kalman_update(state: TiltState, sensor: &SensorData) -> PropagatingEffect<TiltState> {
    // Prediction Setup
    // x_pred: current gravity estimate from prev state
    let gravity_body = match &state.gravity_body {
        Some(g) => g,
        None => return PropagatingEffect::pure(state),
    };
    let x_pred = CausalTensor::new(
        vec![
            gravity_body.get(1).cloned().unwrap_or(0.0),
            gravity_body.get(2).cloned().unwrap_or(0.0),
            gravity_body.get(3).cloned().unwrap_or(0.0),
        ],
        vec![3, 1],
    )
    .unwrap();

    // p_pred: current covariance (no distinct prediction step for P in this model, just take P_prev)
    let p_pred = match &state.covariance {
        Some(p) => p.clone(),
        None => return PropagatingEffect::pure(state),
    };

    // Process Noise Q
    let q = CausalTensor::new(
        vec![Q_DIAG, 0.0, 0.0, 0.0, Q_DIAG, 0.0, 0.0, 0.0, Q_DIAG],
        vec![3, 3],
    )
    .unwrap();

    // If motion detected, we trust prediction (x_pred) + simple diffusion of P
    if state.motion_detected {
        let p_propagated = &p_pred + &q;
        return PropagatingEffect::pure(TiltState {
            covariance: Some(p_propagated),
            ..state
        });
    }

    // Measurement: accelerometer reading z
    let z = match CausalTensor::new(sensor.accel.to_vec(), vec![3, 1]) {
        Ok(t) => t,
        Err(_) => return PropagatingEffect::pure(state),
    };

    // Measurement Matrix H = Identity (3x3)
    let h = CausalTensor::identity(&[3, 3]).unwrap();

    // Measurement Noise R (Adaptive)
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

    // Execute Kalman Filter Kernel using Wrapper (Update Step)
    let kf_effect = kalman_filter_linear(&x_pred, &p_pred, &z, &h, &r, &q);

    // Bind result back to State
    // PropagatingEffect is a wrapper around CausalEffectPropagationProcess.
    // We can map over the value if it implements Functor, or just access value.
    match kf_effect.value.into_value() {
        Some((x_upd, p_upd)) => {
            let metric = Metric::Euclidean(3);
            let g_new = create_vector(x_upd.as_slice(), &metric).unwrap_or_else(|_| {
                // Fallback if vector creation fails (which shouldn't happen)
                let mut data = vec![0.0; 8];
                data[1] = x_upd.as_slice().first().cloned().unwrap_or(0.0);
                data[2] = x_upd.as_slice().get(1).cloned().unwrap_or(0.0);
                data[3] = x_upd.as_slice().get(2).cloned().unwrap_or(0.0);
                CausalMultiVector::new(data, metric).unwrap()
            });

            PropagatingEffect::pure(TiltState {
                gravity_body: Some(g_new),
                covariance: Some(p_upd),
                ..state
            })
        }
        None => PropagatingEffect::from_error(deep_causality::CausalityError(
            deep_causality::CausalityErrorEnum::Custom("Kalman Filter calculation failed".into()),
        )),
    }
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
