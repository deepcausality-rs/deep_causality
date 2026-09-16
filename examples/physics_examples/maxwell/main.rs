/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Maxwell's Unification: E and B as one bivector
//!
//! Electric and magnetic fields are ordinarily two vectors kept mutually consistent by hand. In
//! geometric algebra they are two blades of a single field bivector `F`, derived from one vector
//! potential `A`.
//!
//! This example takes the plane wave `A_x(t, z) = cos(omega (t - z))` and carries it through:
//!
//! ```text
//! A_x            the potential, written once over `Scalar`
//! E_x = -dA_x/dt the electric blade, read off the tangent functor
//! B_y =  dA_x/dz the magnetic blade, from the same sweep
//! F              both blades in one Cl(1,3) bivector
//! S = E x B      the Poynting flux, as an outer product
//! ```
//!
//! `E` and `B` are never hand-differentiated: the potential is evaluated at `Dual` and the
//! partials come out of the epsilon channel exactly. The run then checks the identities a
//! source-free plane wave must satisfy, `d_mu A^mu = 0`, `|E| = |B|` and `|S| = |E||B|`, and
//! checks the AD result against the closed form. A failed identity is a failed run: `main`
//! returns the error and the process exits with a nonzero status.
//!
//! The frequency rides in as a coordinate of the point the potential is evaluated at, next to
//! `t` and `z`. That keeps one `omega` in the program: the value the phase is computed from is
//! the value the wave is differentiated at, at whatever precision the alias below names.
//!
//! ## APIs Demonstrated
//! - `DifferentiateFieldExt::gradient` (forward-mode AD over a scalar-generic field)
//! - `MaxwellSolver::calculate_potential_divergence` and `calculate_poynting_flux`
//! - `CausalMultiVector` blade layout in `Cl(1,3)`

mod model;
mod utils_print;

use deep_causality::{CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_algebra::Real;
use deep_causality_core::{CausalEffect, CausalFlow};
use deep_causality_num::{Float106, const_scalar_from_float, const_scalar_from_int, lift_usize};
use model::{MaxwellState, PlaneWaveConfig};
use utils_print::{print_config, print_fields, print_header, print_verification};

/// Angular frequency of the wave. A whole number, so it is exact at every scalar.
const OMEGA: FloatType = const_scalar_from_int!(FloatType, 1);
/// Observation event `(t, z)`.
const OBSERVE_T: FloatType = const_scalar_from_int!(FloatType, 1);
const OBSERVE_Z: FloatType = const_scalar_from_float!(FloatType, 0.5);

/// How many rounding steps of the working type an identity may miss by and still hold.
///
/// Every quantity here is a closed-form expression evaluated once, so the residuals sit within a
/// few units of the working type's epsilon at any precision. Stating the tolerance in those
/// units is what lets the same check run at `f32`, `f64`, `BFloat16` and `Float106`.
pub const TOLERANCE_ULPS: usize = 64;

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the wave, its
/// exact derivatives, the bivector and the identity checks all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), CausalityError> {
    print_header();

    let config = PlaneWaveConfig {
        omega: OMEGA,
        t: OBSERVE_T,
        z: OBSERVE_Z,
    };
    print_config(&config);

    // Potential -> field bivector -> Poynting flux, as one pipeline.
    let result: PropagatingEffect<MaxwellState> =
        CausalFlow::value(MaxwellState::from_config(&config))
            .bind(|s, _, _| forward(s, model::compute_potential))
            .bind(|s, _, _| forward(s, model::compute_em_field))
            .bind(|s, _, _| forward(s, model::compute_poynting_flux))
            .into_effect();

    // The error channel reaches `main`: a stage that failed is the error the process exits with.
    let (outcome, _, _, _) = result.into_parts();
    let state = outcome?
        .into_value()
        .ok_or_else(|| custom("the chain finished without a field state"))?;

    let f = model::field_bivector(&state).map_err(custom)?;
    print_fields(&state, model::field_blades(&f));

    let verification = verify(&state);
    print_verification(&verification);
    if verification.holds {
        Ok(())
    } else {
        Err(custom(
            "an identity of the source-free plane wave failed at this precision",
        ))
    }
}

/// Hands the stage its state, or short-circuits when the upstream carried none.
fn forward(
    value: CausalEffect<MaxwellState>,
    stage: impl Fn(MaxwellState) -> PropagatingEffect<MaxwellState>,
) -> PropagatingEffect<MaxwellState> {
    match value.into_value() {
        Some(s) => stage(s),
        None => PropagatingEffect::from_error(custom("the chain carried no state")),
    }
}

/// The identities a source-free plane wave in vacuum must satisfy, each against its closed form.
pub struct Verification {
    /// The tolerance the residuals are held to, at the working type.
    pub tolerance: FloatType,
    /// `|d_mu A^mu|`, zero because `A_x` does not depend on `x`.
    pub gauge_residual: FloatType,
    /// `|E| - |B|`, zero because `-dA_x/dt` and `dA_x/dz` differ only in sign for `f(t - z)`.
    pub field_balance: FloatType,
    /// `|S| - |E||B|`, zero because `E` and `B` are orthogonal.
    pub flux_residual: FloatType,
    /// `E_x` against the closed form `omega sin(omega (t - z))`.
    pub closed_form_residual: FloatType,
    pub holds: bool,
}

fn verify(s: &MaxwellState) -> Verification {
    let tolerance = lift_usize::<FloatType>(TOLERANCE_ULPS) * FloatType::epsilon();

    let gauge_residual = Real::abs(s.divergence);
    let field_balance = Real::abs(s.e_field) - Real::abs(s.b_field);
    let flux_residual = s.poynting_flux - Real::abs(s.e_field) * Real::abs(s.b_field);

    // A_x = cos(omega (t - z)), so E_x = -dA_x/dt = omega sin(omega (t - z)).
    let closed_form = s.omega * Real::sin(s.phase);
    let closed_form_residual = s.e_field - closed_form;

    Verification {
        tolerance,
        gauge_residual,
        field_balance,
        flux_residual,
        closed_form_residual,
        holds: gauge_residual < tolerance
            && Real::abs(field_balance) < tolerance
            && Real::abs(flux_residual) < tolerance
            && Real::abs(closed_form_residual) < tolerance,
    }
}

fn custom(reason: impl Into<String>) -> CausalityError {
    CausalityError(CausalityErrorEnum::Custom(reason.into()))
}
