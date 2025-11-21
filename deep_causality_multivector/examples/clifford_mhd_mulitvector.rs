/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};

fn main() {
    println!("--- PLASMA FUSION SIMULATION: Reactor Confinement Check ---");
    println!("Context: A Stationary Tokamak Reactor (e.g., ITER).");
    println!(
        "Goal: Calculate the Lorentz Force vector to ensure Plasma is confined away from the walls."
    );
    println!(
        "Problem: Relativistic effects in high-energy plasma can alter geometric interactions."
    );
    println!("Solution: Use Geometric Algebra to automatically handle the spacetime signature.\n");

    // Case A: Stationary Plasma Fusion
    // Modeled using Classical Euclidean Geometry (Standard Engineering)
    println!(">> SCENARIO A: Low-Energy Plasma Fusion (Classical Euclidean Metric)");
    calculate_confinement_force(
        Metric::Euclidean(3),
        0, // Current flowing Toroidally (X-axis)
        1, // Magnetic Field applied Poloidally (Y-axis plane)
    );

    println!("\n------------------------------------------------------------\n");

    // Case B: Mobile Relativistic  Plasma Fusion
    // Modeled using Relativistic Minkowski Spacetime (High-Energy Physics)
    println!(">> SCENARIO B: Mobile Relativistic Plasma Fusion  (Minkowski Metric)");
    calculate_confinement_force(
        Metric::Minkowski(4),
        1, // Current flowing Toroidally (X-axis, shifted by 1 for Time dim)
        2, // Magnetic Field applied Poloidally (Y-axis plane)
    );

    print_explenation();
}

/// Calculates the Lorentz Force Density in a Fusion Reactor.
///
/// In Plasma Physics, the force density **F** acting on the fluid is the interaction
/// between the Current Density **J** and the Magnetic Field **B**.
///
/// $$ F = J \cdot B $$
/// (In Geometric Algebra, the contraction of a Vector current and Bivector field).
fn calculate_confinement_force(metric: Metric, toroidal_axis: usize, poloidal_axis: usize) {
    // 1. Setup Reactor Geometry
    let idx_current = 1 << toroidal_axis;
    let idx_field_plane = (1 << toroidal_axis) | (1 << poloidal_axis);
    let idx_force_direction = 1 << poloidal_axis;

    println!(
        "  [Geometry] Current Axis: e_{}, Field Plane: e_{}e_{}",
        toroidal_axis, toroidal_axis, poloidal_axis
    );

    // 2. Plasma Current (J)
    // A strong current flowing around the torus (e.g., 10 MegaAmps scale scaled down)
    let j_val = 10.0;
    let mut j_data = vec![0.0; 1 << metric.dimension()];
    j_data[idx_current] = j_val;
    let j_vec = CausalMultiVector::new(j_data, metric).unwrap();
    println!("  [Input] Plasma Current J: {:.1}", j_val);

    // 3. Confining Magnetic Field (B)
    // A strong magnetic field perpendicular to the current to create pressure.
    let b_val = 2.0; // Tesla
    let mut b_data = vec![0.0; 1 << metric.dimension()];
    b_data[idx_field_plane] = b_val;
    let b_field = CausalMultiVector::new(b_data, metric).unwrap();
    println!("  [Input] Magnetic Field B: {:.1}", b_val);

    // 4. COMPUTE THE PHYSICS: F = J . B
    // This single line works for ANY geometry (Classical, Relativistic, 3D, 4D, 10D).
    let force = j_vec.inner_product(&b_field);

    // 5. Analyze the Result
    let force_val = force.get(idx_force_direction).unwrap();
    println!(
        "  [Output] Lorentz Force F: {:.2} e_{}",
        force_val, poloidal_axis
    );

    // 6. Reactor Safety Check
    // In this coordinate system, we interpret the result relative to the wall.
    // Euclidean (+1) implies standard cross-product direction.
    // Minkowski (-1) implies the metric contraction reversed the vector sense due to spacetime signature.
    if *force_val > 0.0 {
        println!("  => STATUS: Classical behavior. Force pushes +Y.");
    } else {
        println!("  => STATUS: Relativistic signature detected. Force pushes -Y.");
        println!(
            "     (NOTE: In a simulation, this sign flip must be accounted for to prevent wall collision!)"
        );
    }
}

fn print_explenation() {
    println!("\n============================================================");
    println!("WHAT THIS MEANS FOR COMPUTATIONAL PHYSICS:");
    println!("1. Metric Agnosticism: The exact same code 'force = J . B' calculated");
    println!("   the correct geometric result for both Classical and Relativistic systems.");
    println!("   Standard codes require manual 'if/else' logic to handle relativistic sign flips.");
    println!();
    println!("2. Safety: In Plasma Fusion, mixing up coordinate systems or metric signatures");
    println!("   causes 'Magnetic Monopole' errors or incorrect force directions.");
    println!("   Here, the Algebra enforces the laws of physics at the Type Level.");
    println!("============================================================");
    //
    // Furthermore, this architecture supports General Relativistic metrics, paving the
    // way for modeling Magnetohydrodynamics in curved spacetime i.e. for plasma fusion
    // based Space Propulsion systems i.e. Direct Fusion Drives.
}
