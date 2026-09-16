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
use deep_causality_num::{lift, lower};
use model::{GrmhdState, SimulationConfig};

/// Central body: ten solar masses. One solar mass is 1476.6 m in geometric units, so `r_s = 2M`.
const SOLAR_MASS_GEOMETRIC_M: f64 = 1476.6;
const SOLAR_MASSES: f64 = 10.0;
/// The plasma orbits at three Schwarzschild radii.
const RADIUS_IN_RS: f64 = 3.0;
/// Radial extent of the plasma column, in metres.
const COLUMN_LENGTH_M: f64 = 1.0;
/// Plasma current density and confining field, in the code's natural units.
const CURRENT_DENSITY: f64 = 10.0;
/// Confining magnetic field.
const MAGNETIC_FIELD: f64 = 2.0;
/// Tidal acceleration above which the plasma is treated relativistically, in m/s^2 geometric.
const TIDAL_THRESHOLD: f64 = 1e-12;

/// `f64` is the right precision here: the curvature spans `1e-20` in `1/m^4` against fields of
/// order one, and every quantity is a closed-form expression rather than an accumulation.
/// `Float106` changes no reported digit; it would matter if this drove a time integration.
pub type FloatType = f64;

fn main() {
    print_header();

    let mass_geometric =
        lift::<FloatType>(SOLAR_MASSES) * lift::<FloatType>(SOLAR_MASS_GEOMETRIC_M);
    let schwarzschild_radius = lift::<FloatType>(2.0) * mass_geometric;
    let config = SimulationConfig {
        schwarzschild_radius,
        radius: lift::<FloatType>(RADIUS_IN_RS) * schwarzschild_radius,
        column_length: lift(COLUMN_LENGTH_M),
        current_density: lift(CURRENT_DENSITY),
        magnetic_field: lift(MAGNETIC_FIELD),
        tidal_threshold: lift(TIDAL_THRESHOLD),
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
        "  Schwarzschild radius r_s = {:.4e} m  ({SOLAR_MASSES} solar masses)",
        lower(c.schwarzschild_radius)
    );
    println!(
        "  Plasma radius r          = {:.4e} m  ({RADIUS_IN_RS} r_s)",
        lower(c.radius)
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
