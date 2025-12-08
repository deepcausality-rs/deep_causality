/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # MAXWELL'S UNIFICATION: Causaloid Example
//!
//! This example demonstrates how to model Maxwell's electromagnetic field unification
//! using DeepCausality's monadic composition with CausalMultiVector.
//!
//! ## Engineering Value
//!
//! In standard engineering, Electric (E) and Magnetic (B) fields are treated as separate vectors,
//! requiring manual consistency checks. In Geometric Algebra, they are unified into a single
//! Electromagnetic Field Bivector (F) derived from a Vector Potential (A).
//!
//! **Application: 5G/6G Antenna Design (Phased Arrays)**
//! - Simulate the Interference Pattern of the Vector Potential directly on the mesh
//! - Calculate A (4 scalars) is 50% faster than calculating E, B (6 scalars)
//! - Numerically more stable (no divergence cleaning)
//!
//! ## Causal Chain
//!
//! ```text
//! PlaneWaveConfig → Vector Potential (A) → EM Field (F = ∇A) → Gauge Check → Results
//! ```
mod model;

use deep_causality::PropagatingEffect;
use model::{MaxwellState, PlaneWaveConfig};

fn main() {
    println!("--- MAXWELL'S UNIFICATION: Causaloid Example ---");
    println!("Goal: Model E and B field derivation as a causal chain.\n");

    // =========================================================================
    // Part 1: Define the Wave Configuration
    // =========================================================================
    let config = PlaneWaveConfig {
        omega: 1.0,
        t: 1.0,
        z: 0.5,
    };
    println!(
        "Wave Configuration: ω={}, t={}, z={}",
        config.omega, config.t, config.z
    );

    // =========================================================================
    // Part 2: Build and Evaluate the Causal Chain via Monadic Composition
    // =========================================================================
    // Each step in the causal chain is a pure function wrapped in PropagatingEffect
    //
    // The chain represents: Config → Potential → EM Field → Gauge Check

    let initial_state = MaxwellState::from_config(&config);
    println!("Phase: {:.4}", initial_state.phase);

    // Execute the causal chain using monadic bind
    let result: PropagatingEffect<MaxwellState> = PropagatingEffect::pure(initial_state)
        .bind(|state, _, _| {
            // Step 1: Compute Vector Potential
            model::compute_potential(state.into_value().unwrap_or_default())
        })
        .bind(|state, _, _| {
            // Step 2: Derive EM Field F = ∇A
            model::compute_em_field(state.into_value().unwrap_or_default())
        })
        .bind(|state, _, _| {
            // Step 3: Check Lorenz Gauge
            model::check_lorenz_gauge(state.into_value().unwrap_or_default())
        });

    // =========================================================================
    // Part 3: Extract and Display Results
    // =========================================================================
    if result.is_err() {
        eprintln!("Causal chain failed: {:?}", result.error);
        return;
    }

    let final_state = result.value.into_value().unwrap_or_default();

    println!("\n--- Results ---\n");
    println!("Vector Potential A_x: {:.4}", final_state.potential_ax);

    // Part A: Electric and Magnetic Fields
    println!("\n>> Extracted Physical Fields:");
    println!(
        "   Electric Field E (Time-Space Bivector): {:.4}",
        final_state.e_field
    );
    println!(
        "   Magnetic Field B (Space-Space Bivector): {:.4}",
        final_state.b_field
    );

    // Part B: Gauge Check
    println!("\n>> Lorenz Gauge Check:");
    println!("   Divergence: {:.6}", final_state.divergence);
    if final_state.gauge_satisfied {
        println!("   >> SUCCESS: Lorenz Gauge Satisfied (Divergence ≈ 0)");
    } else {
        println!("   >> WARNING: Gauge Broken!");
    }

    // Part C: Physics Verification: |E| = |B| for plane wave
    println!("\n>> Physics Verification:");
    if (final_state.e_field.abs() - final_state.b_field.abs()).abs() < 1e-9 {
        println!("   >> VERIFIED: |E| = |B|. Wave propagating at speed c.");
    } else {
        println!(
            "   >> Difference: |E| - |B| = {:.6}",
            final_state.e_field.abs() - final_state.b_field.abs()
        );
    }

    println!("\n--- Example Complete ---");
}
