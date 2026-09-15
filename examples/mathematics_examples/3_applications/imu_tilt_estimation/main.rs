/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Geometric tilt estimation with an adaptive gravity observer
//!
//! An IMU reports angular rate from a gyroscope and proper acceleration from an accelerometer.
//! Turning those two streams into an attitude estimate rests on one question asked at every step:
//! which way is down?
//!
//! The accelerometer answers it while the vehicle holds still. Under manoeuvre it reads gravity
//! together with linear acceleration, so this estimator carries its own model of gravity in the
//! body frame and updates that model through a Kalman step whose trust in the sensor tracks the
//! conditions the sensor is in:
//!
//! ```text
//! A. prediction    the gyro bivector integrates the orientation rotor forward
//! B. observation   a Kalman update refines gravity in the body frame
//! C. correction    a blended rotor pulls the orientation toward the gravity estimate
//! ```
//!
//! Two guards keep the observation honest. Motion detection holds the measurement update back
//! whenever the accelerometer magnitude departs from `g`, which is the signature of linear
//! acceleration riding on top of gravity. Adaptive measurement noise raises `R` with the gyro
//! magnitude, so the filter leans on its model through a fast rotation and on the sensor once the
//! vehicle settles. The pair separates "I am tilting", which the gyro confirms, from "I am
//! accelerating sideways", which the gyro contradicts.
//!
//! Orientation is a rotor in `Cl(3,0)`: rotations compose as geometric products and stay regular
//! at every attitude. Gravity, its covariance and the Kalman gain are `CausalTensor` values, and
//! `CausalFlow::bind` threads the per-step computation, so the sensor stream folds into a single
//! causal chain carrying its own error channel.
//!
//! The kernel takes raw sensor streams and returns state estimates, so it runs on any platform
//! the crate compiles for.
//!
//! Based on Mohammad Javad Azadi's reaction-wheel unicycle:
//! * <https://www.youtube.com/watch?v=2399A6TRG68>
//! * <https://www.youtube.com/watch?v=RyXit-s4L5k>
//! * <https://iamazadi.github.io/Porta.jl/dev/reactionwheelunicycle.html>

use deep_causality_algebra::Real;
use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_multivector::{
    CausalMultiVector, CausalMultiVectorError, Metric, MultiVector, MultiVectorL2Norm,
};
use deep_causality_num::{lift, lift_count, lower};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

// ======================================================================================
// Tuning constants
// ======================================================================================

/// Process noise `Q`, the diagonal of a 3×3 matrix: the per-step variance of the change in
/// body-frame gravity.
///
/// The manoeuvre sets it. Turning at rate `ω` moves the body-frame gravity vector by `g·ω·dt` in
/// one step, which is `9.81 · 1.0 · 0.01 ≈ 0.098` m/s² here, so `Q ≈ (g·ω·dt)² ≈ 0.01`. Holding
/// `Q` at zero drives the covariance to zero within a few steps and freezes the estimate at the
/// first value it converged to, which is what leaves a tilt estimator tracking a stale horizon.
const Q_DIAG: f64 = 0.01;

/// Base measurement noise `R`: the accelerometer noise variance. Smaller values put the weight on
/// the sensor and larger values put it on the model. Take it from the datasheet or from a bench
/// measurement: `0.01` for a high-quality IMU, up to `1.0` for a consumer part.
const R_BASE: f64 = 0.1;

/// Motion detection threshold in m/s². A reading whose magnitude departs from `G_REF` by more than
/// this carries linear acceleration, and the measurement update stands down for that step.
/// `0.5` is sensitive, `2.0` is permissive, and `f64::MAX` leaves every step to the sensor.
const MOTION_THRESHOLD: f64 = 2.0;

/// Adaptive `R`: the effective noise is `R_BASE · (1 + GYRO_SCALE · |gyro|)`, which lowers the
/// trust placed in the accelerometer while the body is turning quickly. `0.5` adapts mildly, `5.0`
/// aggressively, and `0.0` holds `R` at `R_BASE`.
const GYRO_SCALE: f64 = 2.0;

/// Standard gravity in m/s², adjustable for altitude.
const G_REF: f64 = 9.81;

/// How far the orientation moves toward gravity alignment at each step. Smaller values converge
/// smoothly, larger ones converge quickly: `0.01` is smooth, `0.2` is aggressive.
const TILT_CORRECTION_ALPHA: f64 = 0.1;

/// The initial covariance is `INITIAL_VARIANCE · I`, wide enough that the first observations move
/// the estimate freely.
const INITIAL_VARIANCE: f64 = 100.0;

// ======================================================================================
// Scenario
// ======================================================================================

/// The simulated run: `STEPS` samples at `DT` seconds, stationary except for a roll about the
/// x axis at `GYRO_RATE` rad/s over the step range `[TILT_START, TILT_END)`.
const STEPS: usize = 50;
const DT: f64 = 0.01;
const GYRO_RATE: f64 = 1.0;
const TILT_START: usize = 11;
const TILT_END: usize = 30;

/// The bound the recovered attitude is held to, one degree in radians.
const ATTITUDE_TOLERANCE: f64 = 0.0175;

/// The state vector holds three components, and `Cl(3,0)` holds `2^3` coefficients.
const DIMENSION: usize = 3;
const COEFFICIENTS: usize = 8;
/// Blade indices in `Cl(3,0)`: each axis owns one bit, and a plane owns the bits it spans.
const SCALAR: usize = 0;
const E1: usize = 1;
const E2: usize = 2;
const E3: usize = 4;
const E12: usize = E1 | E2;
const E13: usize = E1 | E3;
const E23: usize = E2 | E3;

/// The working scalar. Sensor readings, the rotor, gravity and the covariance all carry it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let metric = Metric::Euclidean(DIMENSION);

    // The starting state: an identity rotor, gravity pointing down the body z axis, and a wide
    // covariance.
    let initial_state = TiltState {
        orientation: Some(CausalMultiVector::scalar(lift::<FloatType>(1.0), metric)),
        gravity_body: Some(vector(&[lift::<FloatType>(0.0); DIMENSION], metric)?.with_z(G_REF)?),
        covariance: Some(diagonal(lift::<FloatType>(INITIAL_VARIANCE))?),
    };

    // Fold the sensor stream into one causal chain. Each `bind` threads the state forward, and a
    // failed step leaves through the process error channel that `into_parts` hands back.
    let flow = CausalFlow::value((SensorData::default(), initial_state));
    let process = simulate_stream()
        .into_iter()
        .fold(flow, |flow, reading| {
            flow.bind(move |previous, _, _| {
                let stepped = advance(previous.into_value(), &reading, metric);
                match stepped {
                    Ok(next) => PropagatingEffect::pure((reading, next)),
                    Err(e) => PropagatingEffect::from_error(CausalityError::new(
                        CausalityErrorEnum::Custom(e.to_string()),
                    )),
                }
            })
        })
        .into_effect();

    let (outcome, ..) = process.into_parts();
    let (_, final_state) = outcome?
        .into_value()
        .ok_or("the causal chain carries no final value")?;

    let gravity = final_state
        .gravity_body
        .ok_or("the final state carries no gravity estimate")?;
    let orientation = final_state
        .orientation
        .ok_or("the final state carries no orientation")?;

    // A rotor through angle φ carries cos(φ/2) in its scalar part, so the recovered roll reads
    // straight back out of the estimate.
    let scalar = blade(&orientation, SCALAR);
    let recovered = lift::<FloatType>(2.0) * Real::acos(scalar);

    let applied = applied_roll();
    print_gravity(
        blade(&gravity, E1),
        blade(&gravity, E2),
        blade(&gravity, E3),
    );
    print_attitude(scalar, recovered, applied);

    // One degree is the bound an attitude estimate of this kind is held to. What is left inside it
    // is the first-order rotor step and the lag the filter carries through the turn.
    assert!(Real::abs(recovered - applied) < lift::<FloatType>(ATTITUDE_TOLERANCE));

    Ok(())
}

/// One step of the estimator: predict from the gyro, observe with the accelerometer, correct the
/// attitude toward the gravity estimate.
fn advance(
    previous: Option<(SensorData, TiltState)>,
    reading: &SensorData,
    metric: Metric,
) -> Result<TiltState, Box<dyn std::error::Error>> {
    let (_, state) = previous.ok_or("the causal chain carries no previous step")?;
    let orientation = state
        .orientation
        .ok_or("the state carries no orientation")?;
    let gravity = state
        .gravity_body
        .ok_or("the state carries no gravity estimate")?;
    let covariance = state.covariance.ok_or("the state carries no covariance")?;

    let predicted = predict_orientation(&orientation, reading, metric)?;
    let (gravity_next, covariance_next) = observe_gravity(&gravity, &covariance, reading)?;
    let gravity_next = vector(gravity_next.as_slice(), metric)?;
    let corrected = correct_tilt(&predicted, &gravity_next, metric)?;

    Ok(TiltState {
        orientation: Some(corrected),
        gravity_body: Some(gravity_next),
        covariance: Some(covariance_next),
    })
}

/// Step A: the gyro carries the rotor forward.
///
/// `R_new = R_old · exp(−½ Ω dt)`, taken to first order as `R_old · (1 − ½ Ω dt)` and renormalised.
fn predict_orientation(
    orientation: &CausalMultiVector<FloatType>,
    reading: &SensorData,
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, Box<dyn std::error::Error>> {
    let omega = gyro_bivector(&reading.gyro, metric)?;
    let half_step = omega * (lift::<FloatType>(-0.5) * reading.dt);
    let one = CausalMultiVector::scalar(lift::<FloatType>(1.0), metric);

    Ok((orientation.clone() * (one + half_step)).normalize_l2())
}

/// Step B: the Kalman update on gravity in the body frame.
///
/// The measurement model is `z = H·x + v` with `H = I`, because the accelerometer observes the
/// gravity vector directly while the body holds still.
fn observe_gravity(
    gravity: &CausalMultiVector<FloatType>,
    covariance: &CausalTensor<FloatType>,
    reading: &SensorData,
) -> Result<(CausalTensor<FloatType>, CausalTensor<FloatType>), Box<dyn std::error::Error>> {
    let z = CausalTensor::new(reading.accel.clone(), vec![DIMENSION, 1])?;
    let x_pred = CausalTensor::new(
        vec![blade(gravity, E1), blade(gravity, E2), blade(gravity, E3)],
        vec![DIMENSION, 1],
    )?;

    // Process noise, carried whichever branch this step takes.
    let q = diagonal(lift::<FloatType>(Q_DIAG))?;

    // A reading whose magnitude departs from gravity carries linear acceleration, so the step
    // propagates the covariance and leaves the estimate where it is.
    let departure = Real::abs(magnitude(&reading.accel) - lift::<FloatType>(G_REF));
    if departure > lift::<FloatType>(MOTION_THRESHOLD) {
        return Ok((x_pred, covariance + &q));
    }

    // Measurement noise rises with the gyro magnitude, which lowers the weight on the
    // accelerometer through a fast rotation.
    let r_effective = lift::<FloatType>(R_BASE)
        * (lift::<FloatType>(1.0) + lift::<FloatType>(GYRO_SCALE) * magnitude(&reading.gyro));
    let r = diagonal(r_effective)?;

    // Innovation y = z − H·x, innovation covariance S = H·P·Hᵀ + R, gain K = P·S⁻¹.
    let y = &z - &x_pred;
    let s = covariance + &r;
    let s_inv = s.inverse()?;
    let k = CausalTensor::ein_sum(&EinSumOp::mat_mul(covariance.clone(), s_inv))?;

    // State update x = x + K·y, covariance update P = (I − K)·P + Q.
    let correction = CausalTensor::ein_sum(&EinSumOp::mat_mul(k.clone(), y))?;
    let x_updated = &x_pred + &correction;

    let i_minus_k = &diagonal(lift::<FloatType>(1.0))? - &k;
    let p_updated = CausalTensor::ein_sum(&EinSumOp::mat_mul(i_minus_k, covariance.clone()))?;

    Ok((x_updated, &p_updated + &q))
}

/// Step C: pull the attitude toward the gravity estimate.
///
/// The rotor carrying one unit vector `v₁` onto another `v₂` is `(1 + v₂v₁)`, normalised. Here it
/// takes the estimated gravity direction, carried into the world frame by `R g ~R`, onto the
/// reference direction. Blending that rotor with the identity spreads the correction over steps.
fn correct_tilt(
    predicted: &CausalMultiVector<FloatType>,
    gravity: &CausalMultiVector<FloatType>,
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, Box<dyn std::error::Error>> {
    // The reference direction in the world frame, NED, so down is −z for the measured reaction.
    let reference = vector(&[lift::<FloatType>(0.0); DIMENSION], metric)?.with_z(-G_REF)?;

    let world = predicted
        .geometric_product(&gravity.normalize())
        .geometric_product(&predicted.reversion())
        .grade_projection(1);

    let one = CausalMultiVector::scalar(lift::<FloatType>(1.0), metric);
    let alignment = (one.clone() + reference.normalize().geometric_product(&world)).normalize();

    let alpha = lift::<FloatType>(TILT_CORRECTION_ALPHA);
    let blended = (one * (lift::<FloatType>(1.0) - alpha) + alignment * alpha).normalize();

    Ok(blended.geometric_product(predicted).normalize())
}

/// The scenario: level, a roll about the x axis, then level again at the new attitude.
///
/// The accelerometer measures the reaction to gravity in the body frame, so the reading rotates
/// with the body: at roll `φ` it reads `[0, −g·sin φ, −g·cos φ]`. Giving both sensors the same
/// story is what lets the recovered attitude be checked against the roll that produced it.
fn simulate_stream() -> Vec<SensorData> {
    let dt = lift::<FloatType>(DT);
    let rate = lift::<FloatType>(GYRO_RATE);
    let g = lift::<FloatType>(G_REF);
    let mut roll = lift::<FloatType>(0.0);

    (0..STEPS)
        .map(|i| {
            let mut gyro = vec![lift::<FloatType>(0.0); DIMENSION];
            if (TILT_START..TILT_END).contains(&i) {
                gyro[0] = rate;
                roll += rate * dt;
            }
            let accel = vec![
                lift::<FloatType>(0.0),
                -g * Real::sin(roll),
                -g * Real::cos(roll),
            ];

            SensorData { accel, gyro, dt }
        })
        .collect()
}

/// The roll the stream applies over the whole run, which the estimate is checked against.
fn applied_roll() -> FloatType {
    lift::<FloatType>(GYRO_RATE)
        * lift::<FloatType>(DT)
        * lift_count::<FloatType>((TILT_END - TILT_START) as u64)
}

// ========================================================================================
//  Data structures
// ========================================================================================

/// The estimator state carried along the causal chain.
#[derive(Clone, Debug, Default)]
struct TiltState {
    /// The attitude estimate, a rotor taking the body frame to the world frame by `R e ~R`.
    orientation: Option<CausalMultiVector<FloatType>>,

    /// The gravity vector in the body frame, refined at each observation.
    gravity_body: Option<CausalMultiVector<FloatType>>,

    /// The `[3, 3]` covariance of the gravity estimate.
    covariance: Option<CausalTensor<FloatType>>,
}

/// One time step of sensor input.
#[derive(Clone, Debug, Default)]
struct SensorData {
    /// The accelerometer reading: `R⁻¹ g R` plus linear acceleration plus noise.
    accel: Vec<FloatType>,

    /// The gyroscope reading as an angular velocity vector `[wx, wy, wz]`.
    gyro: Vec<FloatType>,

    /// The time step in seconds.
    dt: FloatType,
}

// ========================================================================================
//  Helper functions
// ========================================================================================

/// A 3D vector in the given metric, held at the grade-1 blades of `Cl(3,0)`.
fn vector(
    components: &[FloatType],
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, Box<dyn std::error::Error>> {
    if components.len() != DIMENSION {
        return Err(format!("a vector carries {DIMENSION} components").into());
    }
    let mut data = vec![lift::<FloatType>(0.0); COEFFICIENTS];
    data[E1] = components[0];
    data[E2] = components[1];
    data[E3] = components[2];

    Ok(CausalMultiVector::new(data, metric)?)
}

/// The bivector of an angular velocity, `Ω = wx e₂₃ + wy e₃₁ + wz e₁₂`, which is the dual of the
/// vector in three dimensions and the form the rotor derivative `Ṙ = −½ R Ω` takes.
fn gyro_bivector(
    gyro: &[FloatType],
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, Box<dyn std::error::Error>> {
    if gyro.len() != DIMENSION {
        return Err(format!("a gyro reading carries {DIMENSION} components").into());
    }
    let mut data = vec![lift::<FloatType>(0.0); COEFFICIENTS];
    data[E23] = gyro[0];
    data[E13] = -gyro[1]; // e₃₁ = −e₁₃
    data[E12] = gyro[2];

    Ok(CausalMultiVector::new(data, metric)?)
}

/// A `[3, 3]` matrix carrying one value on its diagonal.
fn diagonal(value: FloatType) -> Result<CausalTensor<FloatType>, Box<dyn std::error::Error>> {
    let mut data = vec![lift::<FloatType>(0.0); DIMENSION * DIMENSION];
    for i in 0..DIMENSION {
        data[i * DIMENSION + i] = value;
    }

    Ok(CausalTensor::new(data, vec![DIMENSION, DIMENSION])?)
}

/// One coefficient of a multivector, read by blade index. Every index used here is inside
/// `Cl(3,0)`, and an index beyond it reads as zero.
fn blade(mv: &CausalMultiVector<FloatType>, index: usize) -> FloatType {
    mv.get(index).copied().unwrap_or(lift::<FloatType>(0.0))
}

/// The Euclidean magnitude of a sensor reading.
fn magnitude(components: &[FloatType]) -> FloatType {
    Real::sqrt(
        components
            .iter()
            .fold(lift::<FloatType>(0.0), |acc, &c| acc + c * c),
    )
}

/// Sets the z component of a grade-1 multivector, which the two reference vectors need.
trait WithZ: Sized {
    fn with_z(self, z: f64) -> Result<Self, CausalMultiVectorError>;
}

impl WithZ for CausalMultiVector<FloatType> {
    fn with_z(self, z: f64) -> Result<Self, CausalMultiVectorError> {
        let metric = self.metric();
        let mut data = self.data().to_vec();
        data[E3] = lift::<FloatType>(z);
        CausalMultiVector::new(data, metric)
    }
}

// ========================================================================================
//  Printing
// ========================================================================================

fn print_header() {
    println!("--- Geometric Tilt Estimator & Adaptive Gravity Observer ---\n");
    println!(
        "Scenario: {STEPS} samples at {DT} s, rolling about x at {GYRO_RATE} rad/s \
         over steps {TILT_START}..{TILT_END}.\n"
    );
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_gravity(x: FloatType, y: FloatType, z: FloatType) {
    println!("Final state:");
    println!(
        "  Estimated gravity (body):   [{:7.4}, {:7.4}, {:7.4}] m/s^2",
        lower(x),
        lower(y),
        lower(z)
    );
}

fn print_attitude(scalar: FloatType, recovered: FloatType, applied: FloatType) {
    println!("  Orientation rotor (scalar):  {:.6}", lower(scalar));
    println!("  Roll applied by the stream:  {:.6} rad", lower(applied));
    println!("  Roll read from the rotor:    {:.6} rad", lower(recovered));
    println!(
        "  Absolute error:              {:.2e} rad",
        lower(Real::abs(recovered - applied))
    );
}
