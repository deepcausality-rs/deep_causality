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
//! 4. feedback      CausalTensor        the EM stress-energy T^00 that sources the next cycle
//! 5. analysis      a comparison        which of the two effects dominates
//! ```
//!
//! The curvature is read off the metric rather than stipulated. In the vacuum exterior the Ricci
//! scalar is zero, so the invariant that carries the tidal physics is the Kretschmann scalar
//! `K = 48 M^2 / r^6`, and the run reports both to make that point.
//!
//! ## APIs Demonstrated
//! - `generate_schwarzschild_metric`, `lorentz_force`, `energy_momentum_tensor_em`
//! - `CausalFlow::next` with a stage per solver
//! - A multivector metric chosen at run time from a computed scalar

mod model;

use deep_causality_core::CausalFlow;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lower};
use model::{GrmhdState, SimulationConfig};

/// Central body: ten solar masses. One solar mass is 1476.6 m in geometric units, so `r_s = 2M`.
const SOLAR_MASS_GEOMETRIC_M: FloatType = const_scalar_from_float!(FloatType, 1476.6);
const SOLAR_MASSES: FloatType = const_scalar_from_int!(FloatType, 10);
/// The plasma orbits at three Schwarzschild radii.
const RADIUS_IN_RS: FloatType = const_scalar_from_int!(FloatType, 3);
/// Radial extent of the plasma column, in metres.
const COLUMN_LENGTH_M: FloatType = const_scalar_from_int!(FloatType, 1);
/// Plasma current density, in the code's natural units.
const CURRENT_DENSITY: FloatType = const_scalar_from_int!(FloatType, 10);
/// Confining magnetic field.
const MAGNETIC_FIELD: FloatType = const_scalar_from_int!(FloatType, 2);
/// Tidal acceleration above which the plasma is treated relativistically, in m/s^2 geometric.
const TIDAL_THRESHOLD: FloatType = const_scalar_from_float!(FloatType, 1e-12);
/// Two, for `r_s = 2M`.
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

/// `f64` is the right precision here: the curvature spans `1e-20` in `1/m^4` against fields of
/// order one, and every quantity is a closed-form expression rather than an accumulation.
/// `Float106` changes no reported digit; it would matter if this drove a time integration.
pub type FloatType = f64;

fn main() {
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

    CausalFlow::value(GrmhdState::new(&config))
        .next(|s| model::calculate_curvature(s).into())
        .next(|s| model::select_metric(s).into())
        .next(|s| model::calculate_lorentz_force(s).into())
        .next(|s| model::calculate_energy_momentum(s).into())
        .next(|s| model::analyze_stability(s).into())
        .run(print_report, |err| {
            eprintln!("The simulation failed: {err:?}");
        });
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== GRMHD: General Relativistic Magnetohydrodynamics ===");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Units: geometric (G = c = 1), so a mass is a length\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_config(c: &SimulationConfig) {
    println!("Central body and plasma:");
    println!(
        "  Schwarzschild radius r_s = {:.4e} m  ({:.0} solar masses)",
        lower(c.schwarzschild_radius),
        lower(SOLAR_MASSES)
    );
    println!(
        "  Plasma radius r          = {:.4e} m  ({:.0} r_s)",
        lower(c.radius),
        lower(RADIUS_IN_RS)
    );
    println!(
        "  Column length L          = {:.4e} m",
        lower(c.column_length)
    );
    println!(
        "  Current J, field B       = {:.2}, {:.2}\n",
        lower(c.current_density),
        lower(c.magnetic_field)
    );
}

fn print_report(s: GrmhdState) {
    println!("[1] GR solver: curvature from the Schwarzschild metric");
    println!(
        "      M = r_s / 2               = {:.4e} m",
        lower(s.mass_geometric)
    );
    println!(
        "      Ricci scalar R            = {:.4e}        (vacuum exterior, so zero)",
        lower(s.ricci_scalar)
    );
    println!(
        "      Kretschmann K = 48M^2/r^6 = {:.4e} 1/m^4  (non-zero where Ricci is not)",
        lower(s.kretschmann)
    );
    println!(
        "      curvature radius K^-1/4   = {:.4e} m",
        lower(model::curvature_radius(s.kretschmann))
    );
    println!(
        "      tidal stretch 2ML/r^3     = {:.4e} m/s^2  (across the column)",
        lower(s.tidal_acceleration)
    );

    println!("\n[2] Coupling: the computed tide selects the algebra");
    println!(
        "      tide {:.2e} vs threshold {:.2e}",
        lower(s.tidal_acceleration),
        lower(s.config.tidal_threshold)
    );
    println!("      selected metric           = {}", s.metric_label);

    println!("\n[3] MHD solver: Lorentz force in that algebra");
    println!(
        "      F = J ^ B on e_1 ^ e_2    = {:.4}",
        lower(s.lorentz_force)
    );

    println!("\n[4] Feedback: EM stress-energy");
    println!(
        "      T^00                      = {:.4}",
        lower(s.em_energy_density)
    );

    println!("\n[5] Analysis");
    println!("      {}", s.status);

    println!("\nData flow: spacetime geometry -> coupling -> plasma physics -> gravity feedback");
}
