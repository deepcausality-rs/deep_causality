/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # GRMHD: a tensor solver coupled to a multivector solver
//!
//! Modelling plasma near a compact object couples general relativity, which supplies the
//! spacetime curvature, to magnetohydrodynamics, which supplies the plasma dynamics. The two
//! halves speak different mathematics: curvature lives in a tensor, and the plasma forces live
//! in a Clifford algebra.
//!
//! The point of the example is the seam between them. A quantity the first solver *computes*
//! decides which algebra the second one runs in:
//!
//! ```text
//! 1. GR solver     CausalTensor        Schwarzschild metric -> Kretschmann scalar, tidal stretch
//! 2. coupling      a value decision    the tidal acceleration selects the Clifford metric
//! 3. MHD solver    CausalMultiVector   Lorentz force density F = J ^ B, in that algebra
//! 4. feedback      CausalTensor        the EM stress-energy T^tt on the Schwarzschild metric
//! 5. analysis      a comparison        the curvature the plasma sources against the hole's
//! ```
//!
//! In the vacuum exterior the Ricci scalar is zero, so the invariant that carries the tidal
//! physics is the Kretschmann scalar `K = 48 M^2 / r^6`, and the run reports both to make that
//! point. Every quantity is in geometric units, `G = c = 1`, and every comparison is made in
//! them; the tide is also shown in `m/s^2` for the engineer.
//!
//! Stage 4 crosses the seam in the other direction. The observer measures `B` in an orthonormal
//! frame; the stress-energy kernel works on the coordinate metric `diag(-lapse, 1/lapse, r^2,
//! r^2)`. The run carries `B` into coordinates, lets the kernel produce `T^tt`, and projects
//! that back onto the observer, where it must equal `B^2 / 2`. That identity is the run's check,
//! and a failed check is a failed run: `main` returns the error and the process exits nonzero.
//!
//! ## APIs Demonstrated
//! - `generate_schwarzschild_metric`, `lorentz_force`, `energy_momentum_tensor_em`
//! - `CausalFlow::next` with a stage per solver, and `finish` to reach the error channel
//! - A multivector metric chosen at run time from a computed scalar

mod model;
mod utils_print;

use deep_causality_algebra::Real;
use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum};
use deep_causality_num::{Float106, const_scalar_from_float, const_scalar_from_int, lift_usize};
use model::{GrmhdState, SimulationConfig, frame_energy_density};
use utils_print::{print_config, print_header, print_report, print_verification};

/// Central body: ten solar masses. One solar mass is 1476.6 m in geometric units, so `r_s = 2M`.
const SOLAR_MASS_GEOMETRIC_M: FloatType = const_scalar_from_float!(FloatType, 1476.6);
pub const SOLAR_MASSES: FloatType = const_scalar_from_int!(FloatType, 10);
/// The plasma orbits at three Schwarzschild radii, in the equatorial plane.
pub const RADIUS_IN_RS: FloatType = const_scalar_from_int!(FloatType, 3);
/// Radial extent of the plasma column, in metres.
const COLUMN_LENGTH_M: FloatType = const_scalar_from_int!(FloatType, 1);
/// Plasma current density, in geometric units.
const CURRENT_DENSITY: FloatType = const_scalar_from_int!(FloatType, 10);
/// Confining magnetic field as the static observer measures it, in geometric units.
const MAGNETIC_FIELD: FloatType = const_scalar_from_int!(FloatType, 2);
/// Tidal acceleration above which the plasma is treated relativistically, in `1/m`. Times `c^2`
/// that is about `9e4 m/s^2` across the column.
const TIDAL_THRESHOLD: FloatType = const_scalar_from_float!(FloatType, 1e-12);
/// Two, for `r_s = 2M`.
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

/// How many rounding steps of the working type the energy-density check may miss by.
///
/// The check runs a 4x4 inverse and three matrix products over a metric whose entries span ten
/// orders of magnitude, so a handful of rounding steps is what to expect at any precision.
pub const TOLERANCE_ULPS: usize = 64;

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the metric,
/// the curvature, the Clifford algebra, the stress-energy and the check all recompute at that
/// precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), CausalityError> {
    print_header();

    let mass_geometric = SOLAR_MASSES * SOLAR_MASS_GEOMETRIC_M;
    let schwarzschild_radius = TWO * mass_geometric;
    let config = SimulationConfig {
        schwarzschild_radius,
        radius: RADIUS_IN_RS * schwarzschild_radius,
        column_length: COLUMN_LENGTH_M,
        current_density: CURRENT_DENSITY,
        magnetic_field: MAGNETIC_FIELD,
        tidal_threshold: TIDAL_THRESHOLD,
    };
    print_config(&config);

    // Five stages, one per solver or decision. `finish` hands back the state or the error the
    // chain short-circuited with, so a failed stage is the error the process exits with.
    let state = CausalFlow::value(GrmhdState::new(&config))
        .next(|s| model::calculate_curvature(s).into())
        .next(|s| model::select_metric(s).into())
        .next(|s| model::calculate_lorentz_force(s).into())
        .next(|s| model::calculate_energy_momentum(s).into())
        .next(|s| model::analyze_stability(s).into())
        .finish()?;
    print_report(&state);

    let verification = verify(&state);
    print_verification(&verification);
    if verification.holds {
        Ok(())
    } else {
        Err(CausalityError(CausalityErrorEnum::Custom(
            "the frame energy density departed from B^2/2 at this precision".into(),
        )))
    }
}

/// The identity stage 4 must satisfy: the observer's energy density is `B^2 / 2`.
pub struct Verification {
    /// The tolerance the residual is held to, at the working type.
    pub tolerance: FloatType,
    /// `|rho_EM - B^2/2| / (B^2/2)`.
    pub energy_residual: FloatType,
    pub holds: bool,
}

fn verify(s: &GrmhdState) -> Verification {
    let tolerance = lift_usize::<FloatType>(TOLERANCE_ULPS) * FloatType::epsilon();
    let closed_form = frame_energy_density(s.config.magnetic_field);
    let energy_residual = Real::abs(s.em_energy_density - closed_form) / closed_form;

    Verification {
        tolerance,
        energy_residual,
        holds: energy_residual < tolerance,
    }
}
