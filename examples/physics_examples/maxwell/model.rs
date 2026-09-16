/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The stages of the Maxwell chain, each a pure `MaxwellState -> PropagatingEffect<MaxwellState>`.
//!
//! Blade indices follow the multivector crate's convention: a blade's index **is** the bitmask of
//! the basis vectors spanning it, so the grade is its population count (see
//! `CausalMultiVector::grade_projection`). With `Cl(1,3)` over `(e_t, e_x, e_y, e_z)` that gives
//! the vectors at 1, 2, 4, 8 and the bivectors at the two-bit indices.

use crate::FloatType;
use deep_causality::{CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};
use deep_causality_physics::MaxwellSolver;

/// `Cl(1,3)` holds `2^4` coefficients indexed by bitmask over `(e_t, e_x, e_y, e_z)`.
const COEFFICIENTS: usize = 16;
const E_T: usize = 1;
const E_X: usize = 2;
const E_Y: usize = 4;
const E_Z: usize = 8;
/// The electric blade `e_t ^ e_x`: `E_x` is the `t,x` component of the field bivector.
const E_TX: usize = E_T | E_X;
/// The magnetic blade `e_z ^ e_x`: `B_y = F_zx` by `B_i = (1/2) eps_ijk F_jk`.
const E_ZX: usize = E_Z | E_X;

/// Zero at the working type, for the blades a multivector leaves empty.
pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);

/// The gauge condition is called satisfied below this absolute divergence.
const GAUGE_TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-9);

/// The spacetime metric every multivector in this example carries.
pub fn metric() -> Metric {
    Metric::Minkowski(4)
}

/// Angular frequency of the wave. A whole number, so it is exact at every scalar.
pub const OMEGA: FloatType = const_scalar_from_int!(FloatType, 1);
/// The same frequency as an `f64` literal, for the generic field below: `run<S>` works at a
/// scalar the caller names, which the working type's constant cannot reach.
const OMEGA_LITERAL: f64 = 1.0;

/// The plane-wave vector potential `A_x(t, z) = cos(omega (t - z))`, written once over `Scalar`.
///
/// Evaluated at the working type it is the potential; evaluated at `Dual` it is the potential
/// and its partial derivative, which is where `E` and `B` come from below. No `-omega sin(phase)`
/// is ever written by hand. The struct holds no data: `omega` is a configuration literal lifted
/// per scalar, so nothing here pins a precision.
pub struct PlaneWavePotential;

impl DifferentiableField<2> for PlaneWavePotential {
    fn run<S: Scalar>(&self, tz: &[S; 2]) -> S {
        let omega = deep_causality_num::lift::<S>(OMEGA_LITERAL);
        (omega * (tz[0] - tz[1])).cos()
    }
}

/// Configuration for a plane wave in spacetime.
#[derive(Clone, Debug, Default)]
pub struct PlaneWaveConfig {
    pub omega: FloatType,
    pub t: FloatType,
    pub z: FloatType,
}

/// State threaded through the causal chain.
#[derive(Clone, Debug, Default)]
pub struct MaxwellState {
    pub omega: FloatType,
    pub t: FloatType,
    pub z: FloatType,
    pub phase: FloatType,
    pub potential_ax: FloatType,
    /// `E_x = -dA_x/dt`.
    pub e_field: FloatType,
    /// `B_y = dA_x/dz`.
    pub b_field: FloatType,
    pub divergence: FloatType,
    pub poynting_flux: FloatType,
    pub gauge_satisfied: bool,
}

impl MaxwellState {
    pub fn from_config(config: &PlaneWaveConfig) -> Self {
        Self {
            omega: config.omega,
            t: config.t,
            z: config.z,
            ..Default::default()
        }
    }
}

/// Stage 1: the vector potential `A = (0, A_x, 0, 0)` with `A_x = cos(omega (t - z))`.
///
/// This is where the potential is computed; nothing upstream has evaluated it yet.
pub fn compute_potential(input: MaxwellState) -> PropagatingEffect<MaxwellState> {
    let field = PlaneWavePotential;
    let phase = input.omega * (input.t - input.z);
    let potential_ax = field.run(&[input.t, input.z]);

    PropagatingEffect::pure(MaxwellState {
        phase,
        potential_ax,
        ..input
    })
}

/// Stage 2: the field bivector `F` and the gauge scalar.
///
/// `E` and `B` are the partials of `A_x`, read off the tangent functor:
/// `E_x = -dA_x/dt` and `B_y = dA_x/dz`. They are then placed in the blades they belong to,
/// `e_t ^ e_x` and `e_z ^ e_x`, so `F` is a genuine field bivector rather than a pair of
/// numbers carried alongside one.
pub fn compute_em_field(input: MaxwellState) -> PropagatingEffect<MaxwellState> {
    let field = PlaneWavePotential;
    let [da_dt, da_dz] = field.gradient(&[input.t, input.z]);

    let e_field = -da_dt;
    let b_field = da_dz;

    // The Lorenz gauge scalar, d_mu A^mu. The solver contracts a gradient vector with the
    // potential vector, so the gradient carries the derivative of each component along its own
    // axis: A has only an x component and A_x does not depend on x, so the contraction is zero.
    let gradient_d = match blade_vector(&[(E_T, da_dt), (E_Z, da_dz)]) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    let potential_a = match blade_vector(&[(E_X, input.potential_ax)]) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    let divergence = match MaxwellSolver::calculate_potential_divergence(&gradient_d, &potential_a)
    {
        Ok(d) => d,
        Err(e) => return fail(format!("divergence: {e:?}")),
    };

    PropagatingEffect::pure(MaxwellState {
        e_field,
        b_field,
        divergence,
        ..input
    })
}

/// Stage 3: the Lorenz gauge holds when the divergence vanishes.
pub fn check_lorenz_gauge(input: MaxwellState) -> PropagatingEffect<MaxwellState> {
    let gauge_satisfied = deep_causality_algebra::Real::abs(input.divergence) < GAUGE_TOLERANCE;

    PropagatingEffect::pure(MaxwellState {
        gauge_satisfied,
        ..input
    })
}

/// Stage 4: the Poynting vector `S = E x B`, as the outer product of the two field vectors.
///
/// `E` points along `x` and `B` along `y`, so the two occupy *different* basis vectors and their
/// outer product is the `e_x ^ e_y` bivector whose magnitude is `|E||B|`. Placing them on blades
/// that share a basis vector would make the outer product vanish identically.
pub fn compute_poynting_flux(input: MaxwellState) -> PropagatingEffect<MaxwellState> {
    let e_vec = match blade_vector(&[(E_X, input.e_field)]) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    let b_vec = match blade_vector(&[(E_Y, input.b_field)]) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };

    match MaxwellSolver::calculate_poynting_flux(&e_vec, &b_vec) {
        Ok(s_field) => {
            let poynting_flux = deep_causality_algebra::Real::sqrt(s_field.squared_magnitude());
            PropagatingEffect::pure(MaxwellState {
                poynting_flux,
                ..input
            })
        }
        Err(e) => fail(format!("poynting flux: {e:?}")),
    }
}

/// The field bivector `F = E_x (e_t ^ e_x) + B_y (e_z ^ e_x)`.
///
/// This is the object the whole example is about: one bivector holding both fields, rather than
/// two vectors kept consistent by hand.
pub fn field_bivector(state: &MaxwellState) -> Result<CausalMultiVector<FloatType>, String> {
    blade_vector(&[(E_TX, state.e_field), (E_ZX, state.b_field)])
}

/// The `(e_t ^ e_x, e_z ^ e_x)` blade pair of a field bivector, for reporting.
pub fn field_blades(f: &CausalMultiVector<FloatType>) -> (FloatType, FloatType) {
    let d = f.data();
    (
        d.get(E_TX).copied().unwrap_or(ZERO),
        d.get(E_ZX).copied().unwrap_or(ZERO),
    )
}

/// A multivector with the given coefficients set and every other blade zero.
fn blade_vector(blades: &[(usize, FloatType)]) -> Result<CausalMultiVector<FloatType>, String> {
    let mut data = vec![ZERO; COEFFICIENTS];
    for &(index, value) in blades {
        data[index] = value;
    }
    CausalMultiVector::new(data, metric())
        .map_err(|e| format!("building a Cl(1,3) element failed: {e:?}"))
}

fn fail(reason: impl Into<String>) -> PropagatingEffect<MaxwellState> {
    PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(reason.into())))
}
