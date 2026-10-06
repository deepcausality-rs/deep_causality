/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The stages of the Maxwell chain, each a pure function returning the next `MaxwellState` or the
//! error that stops the chain. The stages that evaluate the potential also read the wave's
//! frequency from the chain's context.
//!
//! Blade indices follow the multivector crate's convention: a blade's index **is** the bitmask of
//! the basis vectors spanning it, so the grade is its population count (see
//! `CausalMultiVector::grade_projection`). With `Cl(1,3)` over `(e_t, e_x, e_y, e_z)` that gives
//! the vectors at 1, 2, 4, 8 and the bivectors at the two-bit indices.

use crate::FloatType;
use deep_causality::{CausalityError, CausalityErrorEnum};
use deep_causality_algebra::Real;
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};
use deep_causality_context::{Context, ContextoidId, Data, NoSpace, NoSpaceTime, NoTime};
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_num::const_scalar_from_int;
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

/// The plane wave as a context: one `Data` contextoid per world fact. The example works in natural
/// units (`c = 1`), which the context's SI spacetime types do not model, so the spatial, temporal
/// and spacetime slots are empty; the observation event rides in the value.
pub type WaveContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: angular frequency of the wave, in natural units (`c = 1`).
pub const OMEGA: ContextoidId = 1;

/// Reads one world fact out of the wave, or the error naming the contextoid id it lacks.
pub fn read(wave: &WaveContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    wave.get_data_by_id(id).ok_or_else(|| {
        custom(format!(
            "the plane wave holds no Datoid with contextoid id {id}"
        ))
    })
}

/// The spacetime metric every multivector in this example carries.
pub fn metric() -> Metric {
    Metric::Minkowski(4)
}

/// Where each argument of [`PlaneWavePotential`] sits.
///
/// The frequency is an argument, with the event. A struct that stored `omega` at one concrete
/// type would have to widen it inside `run`, and the wave would then be evaluated at whatever
/// precision `omega` was written down in, whatever `S` the caller asked for. Passing it in keeps
/// every number in the model at the caller's precision, and it keeps one `omega` in the program:
/// the value the phase is computed from is the value the wave is differentiated at.
pub const AT_T: usize = 0;
pub const AT_Z: usize = 1;
pub const AT_OMEGA: usize = 2;

/// The plane-wave vector potential `A_x(t, z) = cos(omega (t - z))`, written once over `Scalar`.
///
/// Evaluated at the working type it is the potential; evaluated at `Dual` it is the potential
/// and its partial derivative, which is where `E` and `B` come from below. No `-omega sin(phase)`
/// is ever written by hand. The struct holds no data: the frequency rides in the third slot of
/// the point, so nothing here pins a precision.
pub struct PlaneWavePotential;

impl DifferentiableField<3> for PlaneWavePotential {
    fn run<S: Scalar>(&self, at: &[S; 3]) -> S {
        Real::cos(at[AT_OMEGA] * (at[AT_T] - at[AT_Z]))
    }
}

/// State threaded through the causal chain.
#[derive(Clone, Debug, Default)]
pub struct MaxwellState {
    /// The observation event `(t, z)`: the point the chain evaluates the wave at.
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
}

impl MaxwellState {
    /// The chain's starting state: the observation event, with nothing computed yet.
    pub fn at(t: FloatType, z: FloatType) -> Self {
        Self {
            t,
            z,
            ..Default::default()
        }
    }

    /// The point the potential is evaluated and differentiated at: the event, then the frequency.
    fn point(&self, omega: FloatType) -> [FloatType; 3] {
        [self.t, self.z, omega]
    }
}

/// Stage 1: the vector potential `A = (0, A_x, 0, 0)` with `A_x = cos(omega (t - z))`.
///
/// This is where the potential is computed; nothing upstream has evaluated it yet.
pub fn compute_potential(
    input: MaxwellState,
    wave: &WaveContext,
) -> Result<MaxwellState, CausalityError> {
    let omega = read(wave, OMEGA)?;
    let phase = omega * (input.t - input.z);
    let potential_ax = PlaneWavePotential.run(&input.point(omega));

    Ok(MaxwellState {
        phase,
        potential_ax,
        ..input
    })
}

/// Stage 2: the field bivector `F` and the gauge scalar.
///
/// `E` and `B` are the partials of `A_x`, read off the tangent functor:
/// `E_x = -dA_x/dt` and `B_y = dA_x/dz`. They are then placed in the blades they belong to,
/// `e_t ^ e_x` and `e_z ^ e_x`, so `F` is a genuine field bivector. The gradient also carries
/// `dA_x/domega`, which the field equations have no use for; it costs one derivative nobody
/// reads and buys a model with no concrete type written into it.
pub fn compute_em_field(
    input: MaxwellState,
    wave: &WaveContext,
) -> Result<MaxwellState, CausalityError> {
    let omega = read(wave, OMEGA)?;
    let gradient = PlaneWavePotential.gradient(&input.point(omega));
    let da_dt = gradient[AT_T];
    let da_dz = gradient[AT_Z];

    let e_field = -da_dt;
    let b_field = da_dz;

    // The Lorenz gauge scalar, d_mu A^mu. The solver contracts a gradient vector with the
    // potential vector, so the gradient carries the derivative of each component along its own
    // axis: A has only an x component and A_x does not depend on x, so the contraction is zero.
    let gradient_d = blade_vector(&[(E_T, da_dt), (E_Z, da_dz)]).map_err(custom)?;
    let potential_a = blade_vector(&[(E_X, input.potential_ax)]).map_err(custom)?;
    let divergence = MaxwellSolver::calculate_potential_divergence(&gradient_d, &potential_a)
        .map_err(|e| custom(format!("divergence: {e:?}")))?;

    Ok(MaxwellState {
        e_field,
        b_field,
        divergence,
        ..input
    })
}

/// Stage 3: the Poynting vector `S = E x B`, as the outer product of the two field vectors.
///
/// `E` points along `x` and `B` along `y`, so the two occupy *different* basis vectors and their
/// outer product is the `e_x ^ e_y` bivector whose magnitude is `|E||B|`. Placing them on blades
/// that share a basis vector would make the outer product vanish identically.
pub fn compute_poynting_flux(input: MaxwellState) -> Result<MaxwellState, CausalityError> {
    let e_vec = blade_vector(&[(E_X, input.e_field)]).map_err(custom)?;
    let b_vec = blade_vector(&[(E_Y, input.b_field)]).map_err(custom)?;
    let s_field = MaxwellSolver::calculate_poynting_flux(&e_vec, &b_vec)
        .map_err(|e| custom(format!("poynting flux: {e:?}")))?;

    Ok(MaxwellState {
        poynting_flux: Real::sqrt(s_field.squared_magnitude()),
        ..input
    })
}

/// The field bivector `F = E_x (e_t ^ e_x) + B_y (e_z ^ e_x)`.
///
/// This is the object the whole example is about: one bivector holding both fields.
pub fn field_bivector(state: &MaxwellState) -> Result<CausalMultiVector<FloatType>, String> {
    blade_vector(&[(E_TX, state.e_field), (E_ZX, state.b_field)])
}

/// The `(e_t ^ e_x, e_z ^ e_x)` blade pair of a field bivector, for reporting.
pub fn field_blades(f: &CausalMultiVector<FloatType>) -> Result<(FloatType, FloatType), String> {
    let d = f.data();
    match (d.get(E_TX), d.get(E_ZX)) {
        (Some(&e), Some(&b)) => Ok((e, b)),
        _ => Err("the field bivector has no e_t ^ e_x or e_z ^ e_x blade".into()),
    }
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

fn custom(reason: impl Into<String>) -> CausalityError {
    CausalityError(CausalityErrorEnum::Custom(reason.into()))
}
