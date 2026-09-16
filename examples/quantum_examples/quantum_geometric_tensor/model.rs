/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the quantum geometric tensor: the two-band model, one QGT component, and the
//! step that makes the metric dimensionless before transport consumes it.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!`, so the
//! compiler resolves them against the alias in `main`.
//!
//! # Units
//!
//! One system throughout: **energies in meV, lengths in nm, velocity matrix elements in meV·nm**.
//! The velocity operator is `v_i = ∂H/∂k_i`, so its matrix elements carry energy × length, which
//! makes `Q_ij` come out in nm² — the quantum metric is an area, and that is what lets it be
//! divided by `a²` further down.
//!
//! A decimal like `0.246` is derived from integers at the working precision rather than written as
//! a literal, because a literal is a `f64` value rounded once and then widened, and the point of
//! the alias is that nothing is rounded at a precision other than the one in force.

use crate::FloatType;
use deep_causality_num::const_scalar_from_int;
use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    PhysicsError, QuantumEigenvector, QuantumVelocity, quantum_geometric_tensor,
};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// The small numbers the model is written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
pub const THREE: FloatType = const_scalar_from_int!(FloatType, 3);
pub const TEN: FloatType = const_scalar_from_int!(FloatType, 10);

// =============================================================================
// Axes
// =============================================================================

pub const AXIS_X: usize = 0;
pub const AXIS_Y: usize = 1;

/// The four components of the tensor, in row-major order, so the result keeps a `[2, 2]` shape.
pub const AXIS_PAIRS: [(usize, usize); 4] = [
    (AXIS_X, AXIS_X),
    (AXIS_X, AXIS_Y),
    (AXIS_Y, AXIS_X),
    (AXIS_Y, AXIS_Y),
];

pub const AXIS_NAMES: [&str; 2] = ["x", "y"];

// =============================================================================
// Bands
// =============================================================================

/// The band the run is about: a flat band near the neutrality point.
pub const TARGET_BAND: usize = 0;

/// The number of bands and the size of the basis they are written in.
pub const NUM_BANDS: usize = 2;
pub const BASIS_SIZE: usize = 2;

/// Band energies, in meV. A flat band just above zero and one remote band above it.
///
/// The gap between them is the only energy scale in the problem, and it is what the QGT divides
/// by. A small gap is what makes the quantum metric large, which is the whole reason flat-band
/// systems are interesting.
pub const E_FLAT_MEV: FloatType = const_scalar_from_int!(FloatType, 1);
pub const E_REMOTE_MEV: FloatType = const_scalar_from_int!(FloatType, 10);

/// A perfectly flat band has no dispersion, so its conventional band curvature is zero. That is
/// the premise of the comparison the run ends with.
pub const FLAT_BAND_CURVATURE: FloatType = ZERO;

/// The moiré lattice constant, in nm. Graphene's is 0.246 nm.
///
/// It is the length that makes the quantum metric dimensionless, and the length the Drude weight
/// is scaled back up by.
pub fn lattice_constant_nm() -> FloatType {
    const NUMERATOR: FloatType = const_scalar_from_int!(FloatType, 246);
    const DENOMINATOR: FloatType = const_scalar_from_int!(FloatType, 1000);

    NUMERATOR / DENOMINATOR
}

/// The gap the QGT divides by, in meV.
pub fn energy_gap_mev() -> FloatType {
    E_REMOTE_MEV - E_FLAT_MEV
}

/// How much of the gap is added to the denominator to keep a degeneracy from dividing by zero.
///
/// It is a millionth of the squared gap, so it regularises without contributing: a regulator large
/// enough to change the answer is a term in the model rather than a guard against one.
pub fn regularization() -> FloatType {
    const A_MILLIONTH: FloatType = const_scalar_from_int!(FloatType, 1_000_000);
    let gap = energy_gap_mev();

    gap * gap / A_MILLIONTH
}

// =============================================================================
// The model
// =============================================================================

/// A minimal two-band model at one point of the Brillouin zone.
///
/// Everything the QGT needs sits here: where the bands are, what their states are, and how each
/// state moves when the momentum does.
pub struct TwoBandModel {
    pub energies: CausalTensor<FloatType>,
    pub states: QuantumEigenvector<FloatType>,
    /// The velocity operator along each axis, indexed by [`AXIS_X`] and [`AXIS_Y`].
    pub velocity: [QuantumVelocity<FloatType>; 2],
}

/// Builds the model.
///
/// The eigenstates are the basis itself, which keeps the algebra readable: every matrix element
/// below is then an entry of the velocity operator rather than a sum over a change of basis. The
/// geometry comes from the velocity operators, which is where it comes from in a real band
/// structure too.
pub fn two_band_model() -> Result<TwoBandModel, PhysicsError> {
    let energies = CausalTensor::new(vec![E_FLAT_MEV, E_REMOTE_MEV], vec![NUM_BANDS])?;

    let states = QuantumEigenvector::new(CausalTensor::new(
        vec![
            complex(ONE, ZERO),
            complex(ZERO, ZERO),
            complex(ZERO, ZERO),
            complex(ONE, ZERO),
        ],
        vec![BASIS_SIZE, NUM_BANDS],
    )?);

    // The two velocity operators, each off-diagonal and Hermitian, so they connect the flat band
    // to the remote one and to nothing else. Their relative phase is what separates the metric
    // from the curvature: equal phases would leave Q real and the Berry curvature zero.
    let half = ONE / TWO;
    let three_tenths = THREE / TEN;

    let velocity_x = velocity_operator(complex(half, three_tenths))?;
    let velocity_y = velocity_operator(complex(-three_tenths, half))?;

    Ok(TwoBandModel {
        energies,
        states,
        velocity: [velocity_x, velocity_y],
    })
}

/// A velocity operator whose only non-zero entries are the off-diagonal pair `(coupling, conj)`.
fn velocity_operator(
    coupling: Complex<FloatType>,
) -> Result<QuantumVelocity<FloatType>, PhysicsError> {
    let conjugate = complex(coupling.re, -coupling.im);

    Ok(QuantumVelocity::new(CausalTensor::new(
        vec![
            complex(ZERO, ZERO),
            coupling,
            conjugate,
            complex(ZERO, ZERO),
        ],
        vec![BASIS_SIZE, NUM_BANDS],
    )?))
}

/// A complex number at the working precision.
pub fn complex(re: FloatType, im: FloatType) -> Complex<FloatType> {
    Complex::new(re, im)
}

// =============================================================================
// The tensor
// =============================================================================

/// One component of the quantum geometric tensor for the target band.
///
/// ```text
/// Q_ij = Σ_{m ≠ n}  ⟨n|v_i|m⟩ ⟨m|v_j|n⟩ / (E_n − E_m)²
/// ```
///
/// The effect the kernel returns carries either the component or the reason there is none, so the
/// failure is converted here rather than silently dropped at the call site.
pub fn qgt_component(
    model: &TwoBandModel,
    axis_i: usize,
    axis_j: usize,
) -> Result<Complex<FloatType>, PhysicsError> {
    let effect = quantum_geometric_tensor(
        &model.energies,
        &model.states,
        &model.velocity[axis_i],
        &model.velocity[axis_j],
        TARGET_BAND,
        regularization(),
    );

    effect.value_cloned().ok_or_else(|| {
        PhysicsError::NumericalInstability(format!(
            "QGT component ({}, {}) carried no value",
            AXIS_NAMES[axis_i], AXIS_NAMES[axis_j]
        ))
    })
}

/// The quantum metric, the real part of the tensor.
pub fn metric_of(q: Complex<FloatType>) -> FloatType {
    q.re
}

/// The Berry curvature, `Ω_ij = −2 Im(Q_ij)`.
pub fn curvature_of(q: Complex<FloatType>) -> FloatType {
    -TWO * q.im
}

/// The quantum metric divided by the cell area, which is what makes it dimensionless.
///
/// `Q_ij` comes out in nm² because the velocity matrix elements carry energy × length. The
/// transport kernel takes the metric in units of the cell, `g̃ = g / a²`, and scales the whole
/// weight back by `a²` at the end. Skipping this division hands the kernel an area where it
/// expects a ratio, and the answer is then wrong by the cell area in both directions at once.
pub fn reduced_metric(metric_nm2: FloatType) -> FloatType {
    let a = lattice_constant_nm();

    metric_nm2 / (a * a)
}
