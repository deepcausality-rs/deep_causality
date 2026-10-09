/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The physics: each cause as a factorization of conditional tables, the fit of a cause to the
//! passive read-out, and what each experiment does in each cause's world.
//!
//! Every world shares one model of a shot. The common phase from the shared Raman laser and its
//! mirror takes four equally spaced values with equal weight. Given the phase, each cloud's atoms
//! read excited with the cloud's fringe (Sorrentino et al., eq. 1). A detection signal reads
//! excited when its own cloud's atoms are excited, when a dark cloud's background light fires it,
//! or when light leaking in from the other cloud fires it. A leak carries the other cloud's whole
//! population, so it depends on the shared phase through that cloud's fringe. Rotation about a
//! horizontal axis adds the centrifugal phase `2 b k_eff T² Ω²` to the differential phase
//! (Lellouch & Holynski, arXiv:2504.11119, appendix B).
//!
//! A cause is fitted to the passive read-out through its fringe's two centres and its
//! differential phase: three numbers for the three a joint of two binary read-outs carries. The
//! fit is exact, so the passive read-out cannot tell the causes apart. An ellipse cannot tell a
//! phase from its negative, so the fit returns the phase in `[0, π]`.

use crate::constants::{
    ATOMS_A, ATOMS_B, PHASE_POINTS, ROTATION, SHARED_PHASE, SIGNAL_A, SIGNAL_B,
};
use crate::model_types::{Cause, Cloud, Fringe, Instrument, Mechanism, Setting};
use crate::{C, FloatType};
use deep_causality_algebra::Real;
use deep_causality_num::{One, Zero, lift_count};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    CjFactor, FactorSupports, Hypothesis, ProcessFactors, QuantumError, Response, ResponseModel,
    embed_on_legs,
};
use deep_causality_tensor::CausalTensor;
use std::collections::{BTreeMap, BTreeSet};

// =============================================================================
// Conditional tables
// =============================================================================

/// The dimension of a leg: the shared phase takes four values, every other node two.
fn leg_dim(leg: usize) -> usize {
    if leg == SHARED_PHASE { PHASE_POINTS } else { 2 }
}

/// The value of the shared phase at its point `point`, in rad.
fn phase_at(point: usize) -> FloatType {
    let two_pi = FloatType::pi() + FloatType::pi();
    two_pi * lift_count::<FloatType>(point as u64) / lift_count::<FloatType>(PHASE_POINTS as u64)
}

/// A diagonal factor over `legs`, ascending with the first most significant, whose entry at each
/// assignment of the legs is `probability(assignment)`.
fn conditional<F>(legs: &[usize], probability: F) -> CausalTensor<C>
where
    F: Fn(&BTreeMap<usize, usize>) -> FloatType,
{
    let dims: Vec<usize> = legs.iter().map(|&l| leg_dim(l)).collect();
    let d: usize = dims.iter().product();
    let zero = Complex::new(FloatType::zero(), FloatType::zero());
    let mut data = vec![zero; d * d];
    for (index, entry) in (0..d).map(|i| (i, i * d + i)) {
        let mut rest = index;
        let mut assignment = BTreeMap::new();
        for (&leg, &dim) in legs.iter().zip(&dims).rev() {
            assignment.insert(leg, rest % dim);
            rest /= dim;
        }
        data[entry] = Complex::new(probability(&assignment), FloatType::zero());
    }
    CausalTensor::from_slice(&data, &[d, d])
}

/// The probability that a binary node takes `value` when it reads one with probability `p`.
fn bernoulli(p: FloatType, value: usize) -> FloatType {
    if value == 1 { p } else { FloatType::one() - p }
}

/// The probability that a detection signal reads excited, its own atoms `own` and the light
/// leaking in firing it with probability `leak`.
fn fired(instrument: &Instrument, own: usize, leak: FloatType) -> FloatType {
    let own_dark = if own == 1 {
        FloatType::zero()
    } else {
        FloatType::one() - instrument.dark_background
    };
    FloatType::one() - own_dark * (FloatType::one() - leak)
}

/// The excited fraction of a cloud whose fringe has `centre` and `phase` at shared-phase point
/// `point`.
fn excitation(
    instrument: &Instrument,
    centre: FloatType,
    phase: FloatType,
    point: usize,
) -> FloatType {
    let half = instrument.contrast / (FloatType::one() + FloatType::one());
    centre - half * (phase_at(point) + phase).cos()
}

/// The centrifugal phase at the rotation node's value `rate`: the operating rate or the step's.
fn rotation_phase(instrument: &Instrument, rate: usize) -> FloatType {
    if rate == 1 {
        instrument.step_rotation_phase
    } else {
        instrument.operating_rotation_phase
    }
}

// =============================================================================
// Causes as factorizations
// =============================================================================

/// The factorization of `cause`. Every cause shares the phase, both clouds' atoms and both
/// detection signals; a leak adds the shared phase to the receiving signal's parents, since the
/// leaking cloud's population depends on it; rotation adds a rotation node as a parent of cloud
/// B, held at the operating rate.
///
/// # Errors
///
/// The factorization's refusal.
pub fn hypothesis(
    cause: &Cause,
    instrument: &Instrument,
) -> Result<Hypothesis<FloatType>, QuantumError> {
    let Fringe {
        centre_a,
        centre_b,
        phase,
    } = cause.fringe;
    let rotates = cause.mechanism == Mechanism::Rotation;
    let leak = instrument.leak;

    let atoms_b: Vec<usize> = if rotates {
        vec![ATOMS_B, SHARED_PHASE, ROTATION]
    } else {
        vec![ATOMS_B, SHARED_PHASE]
    };
    let signal_a: Vec<usize> = if cause.mechanism.leaks_b_to_a() {
        vec![ATOMS_A, SIGNAL_A, SHARED_PHASE]
    } else {
        vec![ATOMS_A, SIGNAL_A]
    };
    let signal_b: Vec<usize> = if cause.mechanism.leaks_a_to_b() {
        vec![ATOMS_B, SIGNAL_B, SHARED_PHASE]
    } else {
        vec![ATOMS_B, SIGNAL_B]
    };

    let mut tables: Vec<(usize, Vec<usize>, CausalTensor<C>)> = vec![
        (SHARED_PHASE, vec![SHARED_PHASE], {
            let weight = FloatType::one() / lift_count::<FloatType>(PHASE_POINTS as u64);
            conditional(&[SHARED_PHASE], |_| weight)
        }),
        (ATOMS_A, vec![ATOMS_A, SHARED_PHASE], {
            conditional(&[ATOMS_A, SHARED_PHASE], |m| {
                let x = excitation(instrument, centre_a, FloatType::zero(), m[&SHARED_PHASE]);
                bernoulli(x, m[&ATOMS_A])
            })
        }),
        (ATOMS_B, atoms_b.clone(), {
            conditional(&atoms_b, |m| {
                let rotation = m
                    .get(&ROTATION)
                    .map_or(FloatType::zero(), |&rate| rotation_phase(instrument, rate));
                let x = excitation(instrument, centre_b, phase + rotation, m[&SHARED_PHASE]);
                bernoulli(x, m[&ATOMS_B])
            })
        }),
        (SIGNAL_A, signal_a.clone(), {
            conditional(&signal_a, |m| {
                let leaked = m.get(&SHARED_PHASE).map_or(FloatType::zero(), |&point| {
                    leak * excitation(instrument, centre_b, phase, point)
                });
                bernoulli(fired(instrument, m[&ATOMS_A], leaked), m[&SIGNAL_A])
            })
        }),
        (SIGNAL_B, signal_b.clone(), {
            conditional(&signal_b, |m| {
                let leaked = m.get(&SHARED_PHASE).map_or(FloatType::zero(), |&point| {
                    leak * excitation(instrument, centre_a, FloatType::zero(), point)
                });
                bernoulli(fired(instrument, m[&ATOMS_B], leaked), m[&SIGNAL_B])
            })
        }),
    ];
    if rotates {
        tables.push((
            ROTATION,
            vec![ROTATION],
            conditional(&[ROTATION], |m| bernoulli(FloatType::zero(), m[&ROTATION])),
        ));
    }

    let mut factors = ProcessFactors::new();
    let mut supports = FactorSupports::new();
    for (node, legs, table) in tables {
        factors.insert(node, table);
        supports.declare(node, &legs);
    }
    supports.set_leg_dim(SHARED_PHASE, PHASE_POINTS);
    Hypothesis::structural(cause.name, factors, supports)
}

// =============================================================================
// The passive read-out and the fit
// =============================================================================

/// The instrument that accepts the detection-signal outcomes `accept` marks, `(signal A, signal
/// B)`, on `hypothesis`'s joint space.
fn reading<F>(
    hypothesis: &Hypothesis<FloatType>,
    accept: F,
) -> Result<CausalTensor<C>, QuantumError>
where
    F: Fn(usize, usize) -> bool,
{
    let one = FloatType::one();
    let zero = FloatType::zero();
    let entries: Vec<C> = (0..4)
        .map(|i| Complex::new(if accept(i / 2, i % 2) { one } else { zero }, zero))
        .collect();
    let mut data = vec![Complex::new(zero, zero); 16];
    for (i, entry) in entries.into_iter().enumerate() {
        data[i * 4 + i] = entry;
    }
    embed_on_legs(
        &CausalTensor::from_slice(&data, &[4, 4]),
        &BTreeSet::from([SIGNAL_A, SIGNAL_B]),
        &hypothesis.legs()?,
    )
}

/// The joint of the two detection signals in passive operation, indexed `2·signal A + signal B`.
///
/// # Errors
///
/// The evaluation's.
pub fn passive_joint(hypothesis: &Hypothesis<FloatType>) -> Result<[FloatType; 4], QuantumError> {
    let mut joint = [FloatType::zero(); 4];
    for (i, p) in joint.iter_mut().enumerate() {
        *p = hypothesis.evaluate(&reading(hypothesis, |a, b| 2 * a + b == i)?)?;
    }
    Ok(joint)
}

/// The fringe under which `mechanism` reproduces the passive `joint` exactly.
///
/// With `u` the chance a cloud's atoms are not excited, one minus its centre, `w = C²/8` and
/// `v = w cos φ` the covariance of the two clouds' excitations over the shared phase, the joint
/// fixes `u_A`, `u_B` and `v`. A leak from A into B adds terms in `ε` to B's dark read-out and to
/// both reading dark, each linear in `u_B` and `v` once `u_A` is read off A's own read-out; the
/// mirror holds for a leak from B.
///
/// # Errors
///
/// [`QuantumError::CalculationError`] when no fringe of the instrument's contrast reproduces the
/// joint, or for the two-way leak, which is no candidate.
pub fn fit(
    mechanism: Mechanism,
    joint: &[FloatType; 4],
    instrument: &Instrument,
) -> Result<Fringe, QuantumError> {
    let one = FloatType::one();
    let two = one + one;
    let dark = one - instrument.dark_background;
    let w = instrument.contrast * instrument.contrast / lift_count::<FloatType>(8);
    let both_dark = joint[0] / (dark * dark);
    let a_dark = (joint[0] + joint[1]) / dark;
    let b_dark = (joint[0] + joint[2]) / dark;
    let leak = instrument.leak;

    // (u_receiver, v) from the receiver's dark read-out `q` and both reading dark, the sender's
    // dark chance `u_sender` known.
    let leaked = |u_sender: FloatType, q: FloatType| {
        let m_sender = one - u_sender;
        let (a11, a12) = (one - leak * m_sender, leak);
        let (a21, a22) = (
            u_sender - leak * (m_sender * u_sender - w),
            one + leak * (one - two * m_sender),
        );
        let det = a11 * a22 - a12 * a21;
        (
            (q * a22 - a12 * both_dark) / det,
            (a11 * both_dark - a21 * q) / det,
        )
    };
    let (u_a, u_b, v) = match mechanism {
        Mechanism::Benign | Mechanism::Rotation => (a_dark, b_dark, both_dark - a_dark * b_dark),
        Mechanism::LeakAToB => {
            let (u_b, v) = leaked(a_dark, b_dark);
            (a_dark, u_b, v)
        }
        Mechanism::LeakBToA => {
            let (u_a, v) = leaked(b_dark, a_dark);
            (u_a, b_dark, v)
        }
        Mechanism::LeakBothWays => {
            return Err(QuantumError::CalculationError(
                "the two-way leak is a world, not a candidate, and is not fitted".into(),
            ));
        }
    };
    let cosine = v / w;
    if cosine.abs() > one {
        return Err(QuantumError::CalculationError(format!(
            "{mechanism:?} needs cos φ = {cosine:?} to reproduce the passive read-out; the \
             contrast allows no fringe that does"
        )));
    }
    let rotation = if mechanism == Mechanism::Rotation {
        instrument.operating_rotation_phase
    } else {
        FloatType::zero()
    };
    Ok(Fringe {
        centre_a: one - u_a,
        centre_b: one - u_b,
        phase: cosine.acos() - rotation,
    })
}

/// The differential phase the instrument reads from `joint`: the fringe of the benign cause, the
/// shared phase alone, which is the ellipse fit's reading.
///
/// # Errors
///
/// As [`fit`].
pub fn benign_reading(
    joint: &[FloatType; 4],
    instrument: &Instrument,
) -> Result<FloatType, QuantumError> {
    fit(Mechanism::Benign, joint, instrument).map(|f| f.phase)
}

// =============================================================================
// The experiments in each cause's world
// =============================================================================

/// What each setting does in each cause's world.
///
/// * Passive: nothing; the instrument reads both detection signals excited.
/// * Brightening a cloud: its atoms are all excited, the other cloud's are all dark, and a leak
///   now carries a fully bright or a fully dark cloud; the instrument reads the dark cloud's
///   signal excited. Every replacement keeps the node's support, so the factorization stays
///   valid.
/// * The rotation step: a cause with a rotation node is held at the step's rate; the instrument
///   reads the two signals disagreeing, which depends on the differential phase alone because the
///   shared phase averages out of it.
pub struct GradiometerModel {
    /// The numbers the replacement factors are built from.
    pub instrument: Instrument,
}

impl ResponseModel<FloatType, Setting> for GradiometerModel {
    fn respond(
        &self,
        candidate: &Hypothesis<FloatType>,
        setting: &Setting,
    ) -> Result<Response<FloatType>, QuantumError> {
        let supports = candidate.supports().ok_or_else(|| {
            QuantumError::CalculationError(format!("'{}' is not structural", candidate.name()))
        })?;
        let legs = |node: usize| -> Result<Vec<usize>, QuantumError> {
            supports
                .support(node)
                .map(<[usize]>::to_vec)
                .ok_or_else(|| {
                    QuantumError::DimensionMismatch(format!(
                        "'{}' has no node {node}",
                        candidate.name()
                    ))
                })
        };
        let instrument = &self.instrument;
        match setting {
            Setting::Passive => Ok(Response::Intervention {
                factors: Vec::new(),
                instrument: reading(candidate, |a, b| a == 1 && b == 1)?,
            }),
            Setting::Brighten(cloud) => {
                let (bright, dark, bright_signal, dark_signal) = match cloud {
                    Cloud::A => (ATOMS_A, ATOMS_B, SIGNAL_A, SIGNAL_B),
                    Cloud::B => (ATOMS_B, ATOMS_A, SIGNAL_B, SIGNAL_A),
                };
                let held =
                    |node: usize, value: usize| -> Result<CjFactor<FloatType>, QuantumError> {
                        Ok(conditional(&legs(node)?, |m| {
                            if m[&node] == value {
                                FloatType::one()
                            } else {
                                FloatType::zero()
                            }
                        }))
                    };
                let leaked_in = |signal: usize,
                                 own: usize,
                                 leak: FloatType|
                 -> Result<CjFactor<FloatType>, QuantumError> {
                    Ok(conditional(&legs(signal)?, |m| {
                        bernoulli(fired(instrument, m[&own], leak), m[&signal])
                    }))
                };
                let carries_leak = |signal: usize| {
                    supports
                        .support(signal)
                        .is_some_and(|l| l.contains(&SHARED_PHASE))
                };
                let mut factors = vec![(bright, held(bright, 1)?), (dark, held(dark, 0)?)];
                if carries_leak(dark_signal) {
                    factors.push((dark_signal, leaked_in(dark_signal, dark, instrument.leak)?));
                }
                if carries_leak(bright_signal) {
                    factors.push((
                        bright_signal,
                        leaked_in(bright_signal, bright, FloatType::zero())?,
                    ));
                }
                let read_a = dark_signal == SIGNAL_A;
                Ok(Response::Intervention {
                    factors,
                    instrument: reading(candidate, |a, b| if read_a { a == 1 } else { b == 1 })?,
                })
            }
            Setting::RotationStep => {
                let factors = if supports.support(ROTATION).is_some() {
                    vec![(
                        ROTATION,
                        conditional(&[ROTATION], |m| bernoulli(FloatType::one(), m[&ROTATION])),
                    )]
                } else {
                    Vec::new()
                };
                Ok(Response::Intervention {
                    factors,
                    instrument: reading(candidate, |a, b| a != b)?,
                })
            }
        }
    }
}
