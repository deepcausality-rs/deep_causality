/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Thermalizing an SU(3) lattice gauge field
//!
//! Lattice QCD puts a gauge field on a discrete spacetime and measures physical quantities as
//! ensemble averages over field configurations. This example runs that pipeline end to end on a
//! `4⁴` lattice with the SU(3) gauge group:
//!
//! ```text
//! 1. hot start        a random configuration on a periodic CubicalComplex
//! 2. thermalization   Metropolis sweeps driving the field toward the Boltzmann weight
//! 3. observables      average plaquette, a 2×2 Wilson loop, the Polyakov loop
//! 4. APE smearing     one smoothing step that lifts the long-range signal
//! 5. gradient flow    Wilson flow, searching for the reference scale t₀
//! ```
//!
//! `LatticeGaugeField` carries four parameters: the gauge group, the spacetime dimension, the
//! link element type and the working scalar. Here they are `SU3`, `4`, `Complex<FloatType>` and
//! `FloatType`, so every observable comes back in the precision the alias names.
//!
//! Each run draws a fresh random start, so the numbers move from run to run and the physics
//! holds: the plaquette climbs from near zero toward one as the sweeps proceed.

use deep_causality_num::{Lift, lift, lower};
use deep_causality_num_complex::Complex;
use deep_causality_rand::rng;
use deep_causality_topology::{
    CubicalComplex, FlowMethod, FlowParams, GaugeGroup, LatticeGaugeField, SU3, SmearingParams,
};
use std::sync::Arc;

/// Lattice size `L⁴` and spacetime dimension. Small enough that the example runs in seconds.
const L: usize = 4;
const D: usize = 4;

/// Inverse coupling `β = 2N/g²`, at approximately the physical QCD value.
const BETA: f64 = 6.0;

/// Metropolis thermalization: how many sweeps, how wide the proposal, how often to report.
const THERMAL_SWEEPS: usize = 10;
const PROPOSAL_WIDTH: f64 = 0.2;
const REPORT_EVERY: usize = 2;

/// The 2×2 Wilson loop measures the force between static quarks at separation 2.
const LOOP_EXTENT: usize = 2;

/// The Polyakov loop winds around the periodic time direction, which is dimension 0.
const TIME_DIM: usize = 0;

/// Wilson gradient flow: step size and the flow time the search for `t₀` runs to.
const FLOW_EPSILON: f64 = 0.01;
const FLOW_T_MAX: f64 = 0.2;

/// The working scalar. `f64` suits lattice gauge theory at this size; `Float106` carries a
/// higher-precision Wilson flow run through the same code.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let beta = lift::<FloatType>(BETA);
    print_header(beta);

    // 1. A periodic lattice in every direction, carrying a random SU(3) link on every edge.
    let lattice = Arc::new(CubicalComplex::new([L; D], [true; D]));
    let mut rng = rng();

    print_hot_start();
    let mut field = LatticeGaugeField::<SU3, D, Complex<FloatType>, FloatType>::try_random(
        lattice.clone(),
        beta,
        &mut rng,
    )?;
    print_initial_plaquette(field.try_average_plaquette()?);

    // 2. Metropolis sweeps. The acceptance rate tracks how well the proposal width is tuned.
    print_thermalizing();
    let epsilon = lift::<FloatType>(PROPOSAL_WIDTH);
    for sweep in 1..=THERMAL_SWEEPS {
        let acceptance = field.try_metropolis_sweep(epsilon, &mut rng)?;
        let plaquette = field.try_average_plaquette()?;
        if sweep % REPORT_EVERY == 0 {
            print_sweep(sweep, plaquette, acceptance);
        }
    }

    // 3. Observables read off the thermalized configuration.
    print_measuring();
    print_plaquette(field.try_average_plaquette()?);
    print_wilson_loop(wilson_loop_average(&field)?);

    // The Polyakov loop is the order parameter for confinement: it sits at zero in the confined
    // phase and moves away from zero in the deconfined phase.
    let polyakov = field.try_average_polyakov_loop(TIME_DIM)?;
    print_polyakov(polyakov / SU3::matrix_dim().lift::<FloatType>());

    // 4. APE smearing averages each link with its staples, which damps the UV noise and leaves
    //    the long-range physics standing.
    let smear_params = SmearingParams::ape_default();
    print_smearing(smear_params.alpha);
    let smeared = field.try_smear(&smear_params)?;
    print_smeared_plaquette(smeared.try_average_plaquette()?);

    // 5. Wilson gradient flow smooths the field continuously. The reference scale t₀ is the flow
    //    time where t²·⟨E(t)⟩ reaches 0.3, and it sets the lattice spacing.
    let flow_params = FlowParams::<FloatType> {
        epsilon: lift(FLOW_EPSILON),
        t_max: lift(FLOW_T_MAX),
        method: FlowMethod::RungeKutta3,
    };
    print_flowing(flow_params.t_max);
    print_flow_scale(field.try_find_t0(&flow_params).ok());

    print_footer();
    Ok(())
}

/// The 2×2 Wilson loop averaged over every plane through the centre of the lattice, normalised
/// by the number of planes and the dimension of the SU(3) matrices.
fn wilson_loop_average(
    field: &LatticeGaugeField<SU3, D, Complex<FloatType>, FloatType>,
) -> Result<FloatType, Box<dyn std::error::Error>> {
    let centre = [L / 2; D];
    let mut sum = lift::<FloatType>(0.0);
    let mut planes = 0usize;

    for mu in 0..D {
        for nu in (mu + 1)..D {
            sum += field.try_wilson_loop(&centre, mu, nu, LOOP_EXTENT, LOOP_EXTENT)?;
            planes += 1;
        }
    }

    Ok(sum / (planes.lift::<FloatType>() * SU3::matrix_dim().lift::<FloatType>()))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

/// The display boundary: `f64` appears here and nowhere else.
fn print_header(beta: FloatType) {
    println!("=== DeepCausality Lattice Gauge Simulation ===");
    println!("Lattice: {L}x{L}x{L}x{L}");
    println!("Group:   SU(3)");
    println!("Beta:    {:.2}", lower(beta));
}

fn print_hot_start() {
    println!("\n[1] Initializing Field (Hot Start)...");
}

fn print_initial_plaquette(plaquette: FloatType) {
    println!(
        "Initial Plaquette: {:.6} (Expect ~0.0 for hot start)",
        lower(plaquette)
    );
}

fn print_thermalizing() {
    println!("\n[2] Thermalizing (Metropolis)...");
}

fn print_sweep(sweep: usize, plaquette: FloatType, acceptance: FloatType) {
    println!(
        "    Sweep {:2}/{}: Plaq = {:.6}, Acc = {:.1}%",
        sweep,
        THERMAL_SWEEPS,
        lower(plaquette),
        lower(acceptance) * 100.0
    );
}

fn print_measuring() {
    println!("\n[3] Measuring Observables...");
}

fn print_plaquette(plaquette: FloatType) {
    println!("    Average Plaquette: {:.6}", lower(plaquette));
}

fn print_wilson_loop(loop_average: FloatType) {
    println!("    2x2 Wilson Loop:   {:.6}", lower(loop_average));
}

fn print_polyakov(polyakov: FloatType) {
    println!("    Polyakov Loop:     {:.6}", lower(polyakov));
}

fn print_smearing(alpha: FloatType) {
    println!("\n[4] APE Smearing...");
    println!(
        "    Applying 1 step of APE smearing (alpha={})",
        lower(alpha)
    );
}

fn print_smeared_plaquette(plaquette: FloatType) {
    println!(
        "    Smeared Plaquette: {:.6} (Expect closer to 1.0)",
        lower(plaquette)
    );
}

fn print_flowing(t_max: FloatType) {
    println!("\n[5] Wilson Gradient Flow (Scale Setting)...");
    println!("    Flowing field to t_max = {:.2}...", lower(t_max));
}

fn print_flow_scale(t0: Option<FloatType>) {
    match t0 {
        Some(t0) => println!("    Found t0 scale:    {:.4}", lower(t0)),
        None => {
            println!("    t0 sits beyond t_max on a lattice this small; raise t_max to reach it")
        }
    }
}

fn print_footer() {
    println!("\nSimulation Complete.");
}
