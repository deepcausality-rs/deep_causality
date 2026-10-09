/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The physics: what each systematic does to the reading in each configuration, and the
//! interferometer read-out it becomes.
//!
//! The read-out is the probability `½(1 + C sin δφ)` at mid-fringe, with `δφ = k_eff T² Δg` and
//! `Δg` the reading's offset from the reference: a qubit on the equator, a phase channel
//! `R_z(δφ)`, a dephasing channel of the contrast, and the projector the phase moves most. Each
//! candidate is a mechanism whose own channel is the contrast; the response model answers each
//! configuration with the phase channel of the offset the candidate predicts there, plus the
//! Earth tide at the experiment's time.
//!
//! The responses, per systematic:
//!
//! * Coriolis follows the atoms' transverse velocity, fixed to the sensor head, so a 180° turn
//!   about vertical flips it.
//! * The quadratic Zeeman shift does not depend on the wave vector's sign, so it flips the apparent
//!   gravity when the wave vector reverses (Louchet-Chauvet et al., eq. 2), and it scales with the
//!   coil current `s` as `a s² + b s + c` (Hu et al., eq. 9).
//! * A tilt `θ` of the beam axis reads `−g(1 − cos θ)`, and a deliberate tilt adds to it.
//! * Synchronous mirror vibration is what the accelerometer correction removes.
//! * The wavefront aberration vanishes at zero atom temperature for aberrations added in one plane
//!   (Karcher et al.); the example reads it at the operating temperature and at zero only.
//! * The two-photon light shift is linear in the Rabi frequency at constant pulse area, and its
//!   co-propagating share follows the field (Gauguet et al., eq. 5); the example takes that share
//!   as linear in the field.
//! * Clipping is Coriolis-mediated, so a turn flips it too, and it moves with the cloud's starting
//!   position at 14.2 µGal/mm along east-west (Farah et al.).
//!
//! Every effect except the Zeeman shift is odd in the wave vector, so reversing it leaves them.

use crate::model_types::{Cause, Physics, Session, Systematic, Tide};
use crate::{C, FloatType};
use deep_causality_algebra::Real;
use deep_causality_num::{One, Zero, lift_count};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Axis, Channel, Hypothesis, InterferometerConfiguration, Observable, QuantumError, QuantumPlant,
    QubitOperator, Response, ResponseModel, WaveVector, environment_at, instrument_configuration,
    instrument_model,
};
use deep_causality_tensor::CausalTensor;

/// The offset `cause` produces in `configuration`, in m/s², without the tide.
pub fn offset(
    cause: &Cause,
    configuration: &InterferometerConfiguration<FloatType>,
    physics: &Physics,
) -> FloatType {
    let one = FloatType::one();
    let zero = FloatType::zero();
    let size = cause.size;
    let tilt = if cause.systematic == Systematic::Tilt {
        size
    } else {
        zero
    };
    let projection = -physics.gravity * (one - (tilt + configuration.tilt_offset()).cos());
    let k_sign = match configuration.wave_vector() {
        WaveVector::Up => one,
        WaveVector::Down => -one,
    };
    let heading = configuration.heading().cos();
    let field = configuration.bias_field() / physics.bias_field;
    let rabi = configuration.rabi_frequency() / physics.rabi_frequency;
    let systematic = match cause.systematic {
        Systematic::Coriolis => size * heading,
        Systematic::QuadraticZeeman => k_sign * size * physics.zeeman_shares.scale(field),
        Systematic::Tilt => zero,
        Systematic::MirrorVibration => {
            if configuration.accelerometer_correction() {
                zero
            } else {
                size
            }
        }
        Systematic::Wavefront => {
            if configuration.atom_temperature() > zero {
                size
            } else {
                zero
            }
        }
        Systematic::LightShift => {
            let share = physics.light_shift_field_share;
            size * rabi * ((one - share) + share * field)
        }
        Systematic::Clipping => {
            (size + physics.clipping_slope * configuration.cloud_position()[0]) * heading
        }
    };
    projection + systematic
}

/// The size at which `systematic` produces `offset` in the passive configuration. A tilt only
/// reads low, so for a positive offset its closest size is vertical, which produces none.
pub fn calibrated(systematic: Systematic, offset: FloatType, physics: &Physics) -> FloatType {
    match systematic {
        Systematic::Tilt if offset < FloatType::zero() => {
            (FloatType::one() + offset / physics.gravity).acos()
        }
        Systematic::Tilt => FloatType::zero(),
        _ => offset,
    }
}

/// The Earth tide at `tick` s from the start, in m/s².
pub fn tide_at(tick: u64, physics: &Physics) -> FloatType {
    let tide = physics.tide;
    let two_pi = FloatType::pi() + FloatType::pi();
    tide.amplitude * (two_pi * lift_count::<FloatType>(tick) / tide.period + tide.phase).cos()
}

/// The read-out at offset `delta_g`, in m/s²: `½(1 + C sin(k_eff T² Δg))`.
pub fn read_out(
    delta_g: FloatType,
    phase_per_acceleration: FloatType,
    physics: &Physics,
) -> FloatType {
    let half = FloatType::one() / (FloatType::one() + FloatType::one());
    half * (FloatType::one() + physics.contrast * (phase_per_acceleration * delta_g).sin())
}

/// The physics of every world the run knows: each candidate and the world the observations come
/// from, by name, with the tide each one's prediction carries.
pub struct GravimeterModel {
    /// The numbers the responses are built from.
    pub physics: Physics,
    /// Each world's cause and tide.
    pub worlds: Vec<(Cause, Tide)>,
}

impl ResponseModel<FloatType, Session> for GravimeterModel {
    fn respond(
        &self,
        candidate: &Hypothesis<FloatType>,
        session: &Session,
    ) -> Result<Response<FloatType>, QuantumError> {
        let (cause, tide) = self
            .worlds
            .iter()
            .find(|(c, _)| c.name == candidate.name())
            .ok_or_else(|| {
                QuantumError::CalculationError(format!("no world named '{}'", candidate.name()))
            })?;
        let model = instrument_model(&session.context)?;
        let configuration = instrument_configuration(&session.context)?;
        let tide = match tide {
            Tide::Recorded => environment_at(&session.context, session.tick)
                .ok_or_else(|| {
                    QuantumError::CalculationError(format!(
                        "the context records no reading at tick {}",
                        session.tick
                    ))
                })?
                .earth_tide(),
            Tide::Actual => tide_at(session.tick, &self.physics),
        };
        let delta_g = offset(cause, &configuration, &self.physics) + tide;
        let phase_per_acceleration =
            model.k_eff() * model.interrogation_time() * model.interrogation_time();
        let rotation = QubitOperator::rotation(Axis::Z, phase_per_acceleration * delta_g)?;
        Ok(Response::Channel(Channel::unitary(&rotation)?))
    }
}

// =============================================================================
// The interferometer as a qubit
// =============================================================================

fn real(value: FloatType) -> C {
    Complex::new(value, FloatType::zero())
}

/// A mechanism candidate named `name`: its own channel is the fringe contrast, a dephasing that
/// scales the equator's coherence by `C`.
///
/// # Errors
///
/// The channel's refusal of the Kraus operators.
pub fn hypothesis(name: &str, physics: &Physics) -> Result<Hypothesis<FloatType>, QuantumError> {
    let one = FloatType::one();
    let half = one / (one + one);
    let zero = real(FloatType::zero());
    let keep = real((half * (one + physics.contrast)).sqrt());
    let flip = real((half * (one - physics.contrast)).sqrt());
    let identity = CausalTensor::from_slice(&[keep, zero, zero, keep], &[2, 2]);
    let phase_flip = CausalTensor::from_slice(&[flip, zero, zero, -flip], &[2, 2]);
    Ok(Hypothesis::mechanism(
        name,
        Channel::from_kraus(&[identity, phase_flip])?,
    ))
}

/// The atoms after the first beam splitter, at mid-fringe: `(|0⟩ + i|1⟩)/√2`.
///
/// # Errors
///
/// The plant's refusal of the state.
pub fn plant() -> Result<QuantumPlant<FloatType>, QuantumError> {
    let amplitude = (FloatType::one() / (FloatType::one() + FloatType::one())).sqrt();
    QuantumPlant::from_ket(&CausalTensor::from_slice(
        &[real(amplitude), Complex::new(FloatType::zero(), amplitude)],
        &[2],
    ))
}

/// The output port whose population the offset raises: `(|0⟩ − |1⟩)/√2`.
///
/// # Errors
///
/// The observable's refusal of the state.
pub fn output_port() -> Result<Observable<FloatType, 2>, QuantumError> {
    let amplitude = (FloatType::one() / (FloatType::one() + FloatType::one())).sqrt();
    Observable::from_ket(
        "output port",
        &CausalTensor::from_slice(&[real(amplitude), real(-amplitude)], &[2]),
    )
}
