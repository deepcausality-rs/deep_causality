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
//! partials come out of the epsilon channel exactly. The run then checks the two identities a
//! source-free plane wave must satisfy, `|E| = |B|` and `|S| = |E||B|`, against the closed form.
//!
//! ## APIs Demonstrated
//! - `DifferentiateFieldExt::gradient` (forward-mode AD over a scalar-generic field)
//! - `MaxwellSolver::calculate_potential_divergence` and `calculate_poynting_flux`
//! - `CausalMultiVector` blade layout in `Cl(1,3)`

mod model;

use deep_causality::PropagatingEffect;
use deep_causality_algebra::Real;
use deep_causality_core::CausalFlow;
use deep_causality_num::{const_scalar_from_float, lower};
use model::{MaxwellState, PlaneWaveConfig};

/// Observation event `(t, z)`.
const OBSERVE_T: FloatType = deep_causality_num::const_scalar_from_int!(FloatType, 1);
const OBSERVE_Z: FloatType = const_scalar_from_float!(FloatType, 0.5);

/// How close the identities must hold to count as verified.
const IDENTITY_TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);

/// `f64` is the right precision here: the wave is evaluated at one event from closed-form
/// partials, so the identities below close at machine epsilon at any precision. `Float106`
/// tightens the residuals and changes no reported field value.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let config = PlaneWaveConfig {
        omega: model::OMEGA,
        t: OBSERVE_T,
        z: OBSERVE_Z,
    };
    print_config(&config);

    // Potential -> field bivector -> gauge -> Poynting flux, as one pipeline.
    let result: PropagatingEffect<MaxwellState> =
        CausalFlow::value(MaxwellState::from_config(&config))
            .bind(|s, _, _| forward(s, model::compute_potential))
            .bind(|s, _, _| forward(s, model::compute_em_field))
            .bind(|s, _, _| forward(s, model::check_lorenz_gauge))
            .bind(|s, _, _| forward(s, model::compute_poynting_flux))
            .into_effect();

    let state = match result.into_value() {
        Some(s) => s,
        None => {
            println!("The chain produced no field state.");
            return Ok(());
        }
    };

    let f = model::field_bivector(&state)?;
    print_fields(&state, model::field_blades(&f));
    print_verification(&verify(&state));

    Ok(())
}

/// Hands the stage its state, or short-circuits when the upstream carried none.
fn forward(
    value: deep_causality_core::CausalEffect<MaxwellState>,
    stage: impl Fn(MaxwellState) -> PropagatingEffect<MaxwellState>,
) -> PropagatingEffect<MaxwellState> {
    match value.into_value() {
        Some(s) => stage(s),
        None => PropagatingEffect::from_error(deep_causality::CausalityError(
            deep_causality::CausalityErrorEnum::Custom("the chain carried no state".into()),
        )),
    }
}

/// The two identities a source-free plane wave in vacuum must satisfy, each against its closed
/// form rather than against a repeat of the same computation.
struct Verification {
    /// `|E| - |B|`, zero because `-dA_x/dt` and `dA_x/dz` differ only in sign for `f(t - z)`.
    field_balance: FloatType,
    /// `|S| - |E||B|`, zero because `E` and `B` are orthogonal.
    flux_residual: FloatType,
    /// `E_x` against the closed form `omega sin(omega (t - z))`.
    closed_form_residual: FloatType,
    holds: bool,
}

fn verify(s: &MaxwellState) -> Verification {
    let field_balance = Real::abs(s.e_field) - Real::abs(s.b_field);
    let flux_residual = s.poynting_flux - Real::abs(s.e_field) * Real::abs(s.b_field);

    // A_x = cos(omega (t - z)), so E_x = -dA_x/dt = omega sin(omega (t - z)).
    let closed_form = s.omega * Real::sin(s.phase);
    let closed_form_residual = s.e_field - closed_form;

    Verification {
        field_balance,
        flux_residual,
        closed_form_residual,
        holds: Real::abs(field_balance) < IDENTITY_TOLERANCE
            && Real::abs(flux_residual) < IDENTITY_TOLERANCE
            && Real::abs(closed_form_residual) < IDENTITY_TOLERANCE,
    }
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Maxwell's Unification: E and B as one bivector ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_config(c: &PlaneWaveConfig) {
    println!("Plane wave  A_x(t, z) = cos(omega (t - z))");
    println!(
        "Observed at omega = {}, t = {}, z = {}\n",
        lower(c.omega),
        lower(c.t),
        lower(c.z)
    );
}

fn print_fields(s: &MaxwellState, blades: (FloatType, FloatType)) {
    println!("--- The potential and the field it generates ---");
    println!("  phase   omega (t - z)   = {:.6}", lower(s.phase));
    println!("  A_x     cos(phase)      = {:.6}", lower(s.potential_ax));
    println!("  E_x     -dA_x/dt        = {:.6}   [AD]", lower(s.e_field));
    println!("  B_y      dA_x/dz        = {:.6}   [AD]", lower(s.b_field));

    println!("\n--- Both fields in one Cl(1,3) bivector F ---");
    println!("  F on e_t ^ e_x  (E_x)   = {:.6}", lower(blades.0));
    println!("  F on e_z ^ e_x  (B_y)   = {:.6}", lower(blades.1));

    println!("\n--- Gauge and flux ---");
    println!("  d_mu A^mu               = {:.3e}", lower(s.divergence));
    println!(
        "  Lorenz gauge            = {}",
        if s.gauge_satisfied {
            "satisfied"
        } else {
            "BROKEN"
        }
    );
    println!("  |S| = |E x B|           = {:.6}", lower(s.poynting_flux));
}

fn print_verification(v: &Verification) {
    println!("\n--- Identities for a source-free plane wave ---");
    println!(
        "  |E| - |B|               = {:.2e}   (the wave travels at c)",
        lower(v.field_balance)
    );
    println!(
        "  |S| - |E||B|            = {:.2e}   (E and B are orthogonal)",
        lower(v.flux_residual)
    );
    println!(
        "  E_x - omega sin(phase)  = {:.2e}   (AD against the closed form)",
        lower(v.closed_form_residual)
    );
    println!(
        "  => {}",
        if v.holds {
            "all three hold to tolerance"
        } else {
            "AN IDENTITY FAILED"
        }
    );
}
