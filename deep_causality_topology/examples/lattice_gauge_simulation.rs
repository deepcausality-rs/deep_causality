/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lattice Gauge Field Simulation Example
//!
//! This example demonstrates a complete lattice QCD-like simulation using
//! the `LatticeGaugeField` optimizations.
//!
//! Workflow:
//! 1. Initialize a 4x4x4x4 spacetime lattice with SU(3) gauge group
//! 2. Perform a "Hot Start" (random initialization)
//! 3. Thermalize using Metropolis algorithm
//! 4. Measure physical observables:
//!    - Average Plaquette (Action density)
//!    - 2x2 Wilson Loop (Static quark potential)
//!    - Polyakov Loop (Confinement order parameter)
//! 5. Apply advanced smoothing techniques:
//!    - APE Smearing
//!    - Wilson Gradient Flow (computing t0 scale)

use deep_causality_num::Complex;
use deep_causality_rand::rng;
use deep_causality_topology::{
    FlowParams, GaugeGroup, Lattice, LatticeGaugeField, SU3, SmearingParams,
};
use std::sync::Arc;

// Simulation parameters
const L: usize = 4; // Lattice size L^4 (small for example speed)
const D: usize = 4; // Spacetime dimension
const BETA: f64 = 6.0; // Inverse coupling β = 2N/g² (approx physical QCD)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DeepCausality Lattice Gauge Simulation ===");
    println!("Lattice: {}x{}x{}x{}", L, L, L, L);
    println!("Group:   SU(3)");
    println!("Beta:    {:.2}", BETA);

    // 1. Setup Lattice
    // ----------------
    let shape = [L, L, L, L];
    // All dimensions periodic
    let periodic = [true; D];
    let lattice = Arc::new(Lattice::new(shape, periodic));

    println!("\n[1] Initializing Field (Hot Start)...");
    let mut rng = rng();

    // Create random configuration (Hot Start)
    let mut field = LatticeGaugeField::<SU3, D, Complex<f64>, f64>::try_random(
        lattice.clone(),
        BETA,
        &mut rng,
    )?;

    let initial_plaq = field.try_average_plaquette()?;
    println!(
        "Initial Plaquette: {:.6} (Expect ~0.0 for hot start)",
        initial_plaq
    );

    // 2. Thermalization
    // -----------------
    println!("\n[2] Thermalizing (Metropolis)...");
    let thermal_sweeps = 10; // Keep small for example
    let epsilon = 0.2; // Proposal width

    for i in 1..=thermal_sweeps {
        let acceptance = field.try_metropolis_sweep(epsilon, &mut rng)?;
        let plaq = field.try_average_plaquette()?;
        if i % 2 == 0 {
            println!(
                "    Sweep {:2}/{}: Plaq = {:.6}, Acc = {:.1}%",
                i,
                thermal_sweeps,
                plaq,
                acceptance * 100.0
            );
        }
    }

    // 3. Measurements
    // ---------------
    println!("\n[3] Measuring Observables...");

    // Observable A: Average Plaquette
    let plaq = field.try_average_plaquette()?;
    println!("    Average Plaquette: {:.6}", plaq);

    // Observable B: 2x2 Wilson Loop (Rectangular)
    // Measures force between static quarks at distance r=2
    let mut w_2x2_sum = 0.0;
    let mut count = 0;

    // Sample a few loops from the center of the lattice
    let center = [L / 2; D];
    for mu in 0..D {
        for nu in (mu + 1)..D {
            // 2x2 loop
            let w = field.try_wilson_loop(&center, mu, nu, 2, 2)?;
            w_2x2_sum += w;
            count += 1;
        }
    }
    let w_2x2_avg = w_2x2_sum / (count as f64 * SU3::matrix_dim() as f64);
    println!("    2x2 Wilson Loop:   {:.6}", w_2x2_avg);

    // Observable C: Polyakov Loop
    // Order parameter for confinement
    // In pure gauge theory: 0 = confined, Non-zero = deconfined
    let poly_loop = field.try_average_polyakov_loop(0)?; // Time = dim 0
    // try_average_polyakov_loop returns complex trace average, but here we assume T is f64??
    // Wait, try_average_polyakov_loop returns T.
    // If T=f64, it's already the value.
    println!(
        "    Polyakov Loop:     {:.6}",
        poly_loop / SU3::matrix_dim() as f64
    );

    // 4. Advanced: Smearing
    // ---------------------
    println!("\n[4] APE Smearing...");
    // APE smearing reduces UV noise to reveal long-range physics
    let smear_params = SmearingParams::ape_default();
    println!(
        "    Applying 1 step of APE smearing (alpha={})",
        smear_params.alpha
    );

    // Apply 1 step of smearing (returns new field)
    let smeared_field = field.try_smear(&smear_params)?;
    let smeared_plaq = smeared_field.try_average_plaquette()?;
    println!(
        "    Smeared Plaquette: {:.6} (Expect closer to 1.0)",
        smeared_plaq
    );

    // 5. Advanced: Gradient Flow
    // --------------------------
    println!("\n[5] Wilson Gradient Flow (Scale Setting)...");
    // Gradient flow smooths the field continuously to find the reference scale t0
    // t0 is defined where t^2 * <E(t)> = 0.3

    let flow_params = FlowParams::<f64> {
        epsilon: 0.01,
        t_max: 0.2, // Short flow for example
        method: deep_causality_topology::FlowMethod::RungeKutta3,
    };

    println!("    Flowing field to t_max = {:.2}...", flow_params.t_max);

    // Note: In a real simulation, we'd flow step-by-step and measure E(t)
    // Here we use the helper to search for t0
    match field.try_find_t0(&flow_params) {
        Ok(t0) => println!("    Found t0 scale:    {:.4}", t0),
        Err(_) => {
            println!("    t0 not reached within t_max (expected for small lattice/thermalization)")
        }
    }

    println!("\nSimulation Complete.");
    Ok(())
}
