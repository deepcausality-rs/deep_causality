/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalMonad;
use deep_causality_haft::MonadEffect5;
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

// ======================================================================================
// TUNING CONSTANTS
// ======================================================================================

// --- Process Noise Q ---
// Models uncertainty in the gravity estimate itself (e.g., sensor drift, mounting changes).
// Set to 0.0 for stationary systems. Increase if gravity estimate drifts over time.
// Typical values: 0.0 (stationary) to 0.001 (slowly changing).
const Q_DIAG: f64 = 0.0; // Diagonal value for Q matrix (3x3)

// --- Base Measurement Noise R ---
// Accelerometer noise variance. Lower = trust sensor more. Higher = trust model more.
// Measure this from your sensor's datasheet or empirical tests.
// Typical values: 0.01 (high-quality IMU) to 1.0 (noisy consumer sensor).
const R_BASE: f64 = 0.1;

// --- Motion Detection Threshold ---
// If |accel_magnitude - g| > threshold, assume linear acceleration is present.
// Skip the accel update to avoid corrupting gravity estimate.
// Set to f64::MAX to disable motion detection.
// Typical values: 0.5 (sensitive) to 2.0 (permissive) m/s².
const MOTION_THRESHOLD: f64 = 2.0; // m/s² deviation from expected gravity

// --- Adaptive R Parameters ---
// Scale R based on gyro magnitude to reduce accel trust during rapid rotation.
// R_effective = R_BASE * (1 + GYRO_SCALE * |gyro|)
// Set GYRO_SCALE to 0.0 to disable adaptive R.
// Typical values: 0.5 (mild adaptation) to 5.0 (aggressive).
const GYRO_SCALE: f64 = 2.0; // How much gyro magnitude increases R

// --- Reference Gravity ---
// Standard gravity magnitude. Adjust for altitude if needed.
const G_REF: f64 = 9.81; // m/s²

// --- Tilt Correction Blending ---
// How aggressively to correct orientation toward gravity alignment.
// Lower = smoother but slower convergence. Higher = faster but may oscillate.
// Typical values: 0.01 (smooth) to 0.2 (aggressive).
const TILT_CORRECTION_ALPHA: f64 = 0.1;

// -----------------------------------------------------------------------------------------
// Optimal Estimation of the Gravity Vector
// Based on Mohammad Javad Azadi  Reaction-Wheel Unicycle.
// YT:
// * https://www.youtube.com/watch?v=2399A6TRG68
// * https://www.youtube.com/watch?v=RyXit-s4L5k
// Docs:
// * https://iamazadi.github.io/Porta.jl/dev/reactionwheelunicycle.html
//
// ENGINEERING VALUE:
// 1. Eliminates Gimbal Lock: By using Geometric Algebra (Rotors) instead of Euler angles,
//    we avoid singularities and complex rotation matrices.
// 2. Dynamic Calibration: The Adaptive Gravity Observer uses a Causal Monad to continuously
//    refine its estimate of the gravity vector, effectively "learning" the sensor's
//    orientation relative to gravity in real-time.
// 3. Hardware Independence: This implementation is a Pure Causal Kernel. It takes raw
//    sensor streams and outputs state estimates, making it portable across any robot
//    platform (ARM, x86, RISC-V) without dependency on specific hardware registers.
// -----------------------------------------------------------------------------------------
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Geometric Tilt Estimator & Adaptive Gravity Observer ---");

    // 1. Setup Metric (Euclidean 3D)
    let metric = Metric::Euclidean(3);

    // 2. Initialize State
    // Initial Orientation: Identity (Scalar 1.0)
    let mut init_orientation_data = vec![0.0; 8];
    init_orientation_data[0] = 1.0;
    let init_orientation = CausalMultiVector::new(init_orientation_data, metric)?;

    // Initial Gravity Estimate (Body Frame): Assuming starting flat, gravity is down (-Z or +Z depending on frame).
    // Let's assume NED (North-East-Down), so Gravity is +g in Z axis (down).
    // But accelerometer measures reaction force (up), so reading is -g in Z.
    // Let's stick to: Gravity vector g = [0, 0, 9.81].
    let init_gravity = create_vector(&[0.0, 0.0, 9.81], &metric)?;

    // Initial Covariance P = 100 * I
    let init_covariance = CausalTensor::new(
        vec![100.0, 0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 100.0],
        vec![3, 3],
    )?;

    let initial_state = TiltState {
        orientation: Some(init_orientation),
        gravity_body: Some(init_gravity),
        covariance: Some(init_covariance),
        // forgetting_factor: 0.98, // Not used in the example
    };

    // 3. Create Causal Process (Monad)
    // We carry the state in the Value because CausalMonad's bind preserves the 'S' generic state from the input,
    // effectively making 'S' an immutable context. To evolve state, we pass it in 'T'.
    let process = CausalMonad::<(), ()>::pure((SensorData::default(), initial_state));

    // 4. Simulate Data Stream
    // Scenario: Robot is stationary, then tilts 45 degrees around X axis.
    let steps = 50;
    let dt = 0.01;
    let mut sensor_stream = Vec::new();

    for i in 0..steps {
        let _t = i as f64 * dt;
        let mut gyro = vec![0.0, 0.0, 0.0];
        let accel = vec![0.0, 0.0, -9.81]; // Accelerometer measures proper acceleration (upwards when stationary)

        if i > 10 && i < 30 {
            // Tilting around X axis
            gyro[0] = 1.0; // 1 rad/s
            // If we rotate +X, gravity vector (relative to body) rotates -X.
        }

        sensor_stream.push(SensorData { accel, gyro, dt });
    }

    // 5. Run the Monadic Chain
    // We fold over the stream, binding each step.
    let final_process = sensor_stream
        .into_iter()
        .fold(process, |current_process, input| {
            CausalMonad::bind(current_process, |prev_input| {
                // Propagate the causal chain:
                // 1. `current_process` holds the previous state.
                // 2. `input` is the new sensor reading from the stream.
                // 3. `prev_input` (ignored) is the data from the previous step.

                let (_prev_sensor_data, prev_state) = prev_input;

                // We use the `input` from the fold (current sensor reading)
                // and `prev_state` (from the previous step's value).
                let current_sensor_data = input.clone();
                let dt = current_sensor_data.dt;

                // --- Step A: Prediction (Kinematics) ---
                // R_new = R_old * exp(-0.5 * Omega * dt)
                // Approx: R_new = R_old * (1 - 0.5 * Omega * dt)
                let metric = Metric::Euclidean(3);
                let omega = create_bivector_from_gyro(&current_sensor_data.gyro, &metric).unwrap();
                let half_omega_dt = omega.clone() * (-0.5 * dt);
                // exp(B) approx 1 + B (first order)
                // We need to add scalar 1.0 to Bivector.
                let mut one_data = vec![0.0; 8];
                one_data[0] = 1.0;
                let one = CausalMultiVector::new(one_data, metric).unwrap();

                let rotor_update = one + half_omega_dt;
                let current_orientation = prev_state.orientation.clone().unwrap();
                let predicted_orientation = current_orientation * rotor_update;
                // Normalize rotor
                // R * ~R = 1
                // For now, let's just assume it stays close to unit or implement normalization if API supports it.
                // CausalMultiVector doesn't `normalize()` exposed in the basic API I saw,
                // but we can compute norm and scale.
                // let norm = predicted_orientation.norm_l2(); // This returns a scalar (Tensor or float?)
                // Let's skip normalization for this simple example or assume small steps.

                // --- Step B: Adaptive Gravity Observer (RLS / Kalman Filter) ---
                //
                // -------------------------------------------------------------------------------------
                // WHY THIS MATTERS (For Robotics):
                //
                // 1. THE PROBLEM: "Down" is hard to find.
                //    Accelerometers measure *Proper Acceleration* (Gravity + Motion), not just Gravity.
                //    If a drone banks to turn, the accelerometer reads a huge sideways force.
                //    A naive filter thinks "Down" has shifted sideways, causing the horizon to tilt wrong.
                //
                // 2. CONVENTIONAL APPROACH: Complementary Filter.
                //    "Trust Gyro for fast moves, Trust Accel for slow moves."
                //    Fail Mode: During a long, sustained turn (e.g., orbiting a point), the "slow"
                //    accel error creeps in, and the horizon drifts.
                //
                // 3. THE SOLUTION: Adaptive Gravity Observer.
                //    Instead of blindly trusting the accelerometer, we use a Causal Model.
                //    We *predict* gravity in the body frame based on our orientation.
                //    We then compare this prediction to the sensor reading.
                //    Crucially, we only update our "Gravity Model" slowly (Adaptive RLS/Kalman).
                //
                //    Result: The system distinguishes between "I am tilting" (Gyro confirms)
                //    and "I am accelerating sideways" (Gyro denies).
                //    This gives rock-solid horizon tracking even during high-G maneuvers.
                // -------------------------------------------------------------------------------------

                // We want to estimate g_body (x).
                // Measurement: accel (z).
                // Model: z = H*x + v (Measurement Noise R)
                // Here H = I (Identity), since we measure gravity directly.

                // 1. Setup Matrices
                let z_data = current_sensor_data.accel.clone();
                let z = CausalTensor::new(z_data.clone(), vec![3, 1])
                    .expect("Failed to create z tensor");

                let x_old = prev_state.gravity_body.clone().unwrap();
                let x_old_vec = vec![
                    x_old.get(1).cloned().unwrap_or(0.0),
                    x_old.get(2).cloned().unwrap_or(0.0),
                    x_old.get(3).cloned().unwrap_or(0.0),
                ];
                let x_pred = CausalTensor::new(x_old_vec.clone(), vec![3, 1])
                    .expect("Failed to create x_pred");

                let p_old = prev_state.covariance.clone().unwrap();

                // -------------------------------------------------------------------------------------
                // MOTION DETECTION: Skip accel update if linear acceleration detected
                // -------------------------------------------------------------------------------------
                let accel_magnitude =
                    (z_data[0].powi(2) + z_data[1].powi(2) + z_data[2].powi(2)).sqrt();
                let motion_detected = (accel_magnitude - G_REF).abs() > MOTION_THRESHOLD;

                // -------------------------------------------------------------------------------------
                // ADAPTIVE R: Increase measurement noise when gyro indicates rapid motion
                // -------------------------------------------------------------------------------------
                let gyro = &current_sensor_data.gyro;
                let gyro_magnitude = (gyro[0].powi(2) + gyro[1].powi(2) + gyro[2].powi(2)).sqrt();
                let r_effective = R_BASE * (1.0 + GYRO_SCALE * gyro_magnitude);

                // Construct R matrix with adaptive value
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
                .expect("Failed to create R");

                // -------------------------------------------------------------------------------------
                // PROCESS NOISE Q: Model uncertainty in gravity estimate
                // -------------------------------------------------------------------------------------
                let q = CausalTensor::new(
                    vec![Q_DIAG, 0.0, 0.0, 0.0, Q_DIAG, 0.0, 0.0, 0.0, Q_DIAG],
                    vec![3, 3],
                )
                .expect("Failed to create Q");

                // Conditional update: Only update if no motion detected
                let (x_new, p_new) = if motion_detected {
                    // Skip measurement update, only propagate covariance with process noise
                    let p_propagated = &p_old + &q;
                    (x_pred, p_propagated)
                } else {
                    //  Indirect Kalman Filter (or Error-State Kalman Filter) simplified with Geometric Algebra.

                    // 2. Innovation (Residual): y = z - H*x_pred
                    let y = &z - &x_pred;

                    // 3. Innovation Covariance: S = H*P*H^T + R (with H=I)
                    let s = &p_old + &r;

                    // 4. Kalman Gain: K = P * S^-1
                    let s_inv = s.inverse().expect("S matrix singular");
                    let k = CausalTensor::ein_sum(&EinSumOp::mat_mul(p_old.clone(), s_inv))
                        .expect("MatMul failed");

                    // 5. State Update: x_new = x_pred + K * y
                    let correction = CausalTensor::ein_sum(&EinSumOp::mat_mul(k.clone(), y))
                        .expect("Correction MatMul failed");
                    let x_updated = &x_pred + &correction;

                    // 6. Covariance Update: P_new = (I - K)*P + Q
                    let identity_data = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
                    let identity = CausalTensor::new(identity_data, vec![3, 3])
                        .expect("Failed to create Identity");

                    let i_minus_k = &identity - &k;
                    let p_updated = CausalTensor::ein_sum(&EinSumOp::mat_mul(i_minus_k, p_old))
                        .expect("Covariance Update failed");

                    // Add process noise
                    let p_with_q = &p_updated + &q;

                    (x_updated, p_with_q)
                };

                // Convert back to MultiVector for state
                let x_new_slice = x_new.as_slice();
                let g_new = create_vector(x_new_slice, &metric).unwrap();

                // --- Tilt Correction ---
                // Approximation for small angles (Linear Interpolation + Normalization, or NLERP).
                // -------------------------------------------------------------------------------------
                // HOW IT WORKS:
                // 1. Transform estimated gravity from Body to World: g_world_est = R * g_body * ~R.
                // 2. Compare to reference gravity g_ref = [0, 0, -9.81].
                // 3. Calculate a small correction rotor that aligns g_world_est with g_ref.
                // 4. Apply correction to orientation: R_new = R_correction * R_predicted.
                //
                // The rotor between two unit vectors v1 and v2:
                //   R = (1 + v2*v1) / |1 + v2*v1|  (normalized)
                // This rotates v1 onto v2.
                // -------------------------------------------------------------------------------------

                // Reference gravity in World Frame (NED: Down is +Z)
                let g_ref = create_vector(&[0.0, 0.0, -9.81], &metric).unwrap();

                // Normalized vectors for rotor calculation
                let g_body_norm = g_new.normalize();
                let g_ref_norm = g_ref.normalize();

                // Transform g_body to World Frame: g_world_est = R * g_body * ~R
                let r_rev = predicted_orientation.reversion();
                let temp = predicted_orientation.geometric_product(&g_body_norm);
                let g_world_est = temp.geometric_product(&r_rev).grade_projection(1);

                // Rotor to align g_world_est with g_ref: R_correction = (1 + g_ref * g_world_est), normalized
                // This rotor rotates g_world_est onto g_ref.
                let one = CausalMultiVector::scalar(1.0, metric);
                let alignment_product = g_ref_norm.geometric_product(&g_world_est);
                let rotor_unnorm = one + alignment_product;
                let correction_rotor = rotor_unnorm.normalize();

                // Apply correction: R_corrected = R_correction * R_predicted
                // Use a blending factor (alpha) for smooth correction (SLERP-like)
                let identity_rotor = CausalMultiVector::scalar(1.0, metric);

                // Blend: R_blend = (1-alpha)*I + alpha*R_correction
                // Then normalize to get a valid rotor.
                let weighted_identity = identity_rotor * (1.0 - TILT_CORRECTION_ALPHA);
                let weighted_correction = correction_rotor * TILT_CORRECTION_ALPHA;
                let blended_rotor = (weighted_identity + weighted_correction).normalize();

                // Apply blended correction
                let corrected_orientation = blended_rotor
                    .geometric_product(&predicted_orientation)
                    .normalize();

                let next_state = TiltState {
                    orientation: Some(corrected_orientation),
                    gravity_body: Some(g_new),
                    covariance: Some(p_new),
                };

                // Return new process
                CausalMonad::pure((current_sensor_data, next_state))
            })
        });

    // 6. Output Results
    let final_value = final_process.value.into_value().unwrap();
    let (_, final_state) = final_value;

    println!("Final State:");
    if let Some(g) = final_state.gravity_body {
        println!("  Estimated Gravity (Body): {:?}", g.get(3)); // Z component
    }
    if let Some(r) = final_state.orientation {
        println!("  Orientation Rotor (Scalar): {:?}", r.get(0));
    }

    Ok(())
}

// ========================================================================================
//  Data Structures
// ========================================================================================

/// Represents the state of the Tilt Estimator.
#[derive(Clone, Debug, Default)]
struct TiltState {
    /// Current orientation estimate (Rotor).
    /// Represents rotation from Body frame to World frame (or vice versa depending on convention).
    /// Here: R * e_body * R_rev = e_world
    orientation: Option<CausalMultiVector<f64>>,

    /// Estimated Gravity Vector in Body Frame.
    /// We use RLS to adaptively estimate this.
    gravity_body: Option<CausalMultiVector<f64>>,

    /// Covariance matrix for RLS (P matrix).
    /// Shape: [3, 3]
    covariance: Option<CausalTensor<f64>>,
    // Forgetting factor for RLS (lambda). Not used in this example.
    // forgetting_factor: f64,
}

/// Input sensor data for a single time step.
#[derive(Clone, Debug, Default)]
struct SensorData {
    /// Accelerometer reading (Vector).
    /// Ideally: a = R_inv * g * R + linear_accel + noise
    accel: Vec<f64>,

    /// Gyroscope reading (Bivector components or Angular Velocity Vector).
    /// We'll assume input is angular velocity vector [wx, wy, wz].
    gyro: Vec<f64>,

    /// Time step (dt) in seconds.
    dt: f64,
}

// ========================================================================================
//  Helper Functions
// ========================================================================================

/// Creates a 3D vector in the given metric.
fn create_vector(
    components: &[f64],
    metric: &Metric,
) -> Result<CausalMultiVector<f64>, Box<dyn std::error::Error>> {
    // For Euclidean 3D (Algebra Cl(3,0)):
    // Basis: 1, e1, e2, e3, e12, e13, e23, e123 (Total 8)
    // Indices: 0, 1, 2, 3, 4, 5, 6, 7
    // Vector components are at indices 1, 2, 3.
    let mut data = vec![0.0; 8];
    if components.len() != 3 {
        return Err("Vector must have 3 components".into());
    }
    data[1] = components[0];
    data[2] = components[1];
    data[3] = components[2];
    Ok(CausalMultiVector::new(data, *metric)?)
}

/// Creates a Bivector from angular velocity (dual of vector in 3D).
/// w = wx*e1 + wy*e2 + wz*e3
/// B = I * w = (e1e2e3) * (wx*e1 + wy*e2 + wz*e3)
///   = wx*e23 - wy*e13 + wz*e12
/// Note: Convention varies. Let's use B = -(wx*e23 + wy*e31 + wz*e12) or similar.
/// Standard relation: Rotor derivative R_dot = -0.5 * R * Omega
/// where Omega is the bivector representation of angular velocity.
/// Omega = wx * e23 + wy * e31 + wz * e12 (typically).
fn create_bivector_from_gyro(
    gyro: &[f64],
    metric: &Metric,
) -> Result<CausalMultiVector<f64>, Box<dyn std::error::Error>> {
    // Gyro: [wx, wy, wz]
    // Bivector Omega = wx*e23 + wy*e31 + wz*e12
    // e23 index: 6
    // e31 = -e13 index: 5 (so -wy at index 5)
    // e12 index: 4
    let mut data = vec![0.0; 8];
    if gyro.len() != 3 {
        return Err("Gyro must have 3 components".into());
    }
    let wx = gyro[0];
    let wy = gyro[1];
    let wz = gyro[2];

    data[6] = wx; // e23
    data[5] = -wy; // e13 (e31 = -e13)
    data[4] = wz; // e12

    Ok(CausalMultiVector::new(data, *metric)?)
}
