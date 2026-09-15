/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_num::{lift, lower};

// -----------------------------------------------------------------------------------------
// ENGINEERING VALUE:
// Plasma Physics and Fusion research often involve switching between Classical (Euclidean)
// and Relativistic (Minkowski) regimes. Errors in metric signatures can lead to
// catastrophic simulation failures (e.g., calculating force in the wrong direction).
//
// This example demonstrates "Metric Agnosticism":
// The same code `F = J . B` correctly calculates the Lorentz Force in BOTH regimes.
// The `CausalMultiVector` type handles the underlying metric algebra (spacetime signature),
// ensuring physical correctness and safety at the type level.
// -----------------------------------------------------------------------------------------

/// The working scalar. Current, field and the force they produce all carry it.
pub type FloatType = f64;

fn main() {
    print_header();

    // Case A: a stationary reactor, modelled in classical Euclidean geometry.
    print_scenario("A", "Stationary Plasma Fusion (Classical Euclidean Metric)");
    calculate_confinement_force(
        Metric::Euclidean(3),
        0, // current flowing toroidally (x-axis)
        1, // magnetic field applied poloidally (y-axis plane)
    );

    print_separator();

    // Case B: the same calculation in relativistic Minkowski spacetime. The axis indices
    // shift by one because the time dimension comes first.
    print_scenario("B", "Mobile Relativistic Plasma Fusion  (Minkowski Metric)");
    calculate_confinement_force(Metric::Minkowski(4), 1, 2);

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

    print_geometry(toroidal_axis, poloidal_axis);

    // 2. Plasma current J: a strong current around the torus, on the order of 10 MA.
    let j_val = lift::<FloatType>(10.0);
    let j_vec = blade(idx_current, j_val, metric);

    // 3. Confining magnetic field B, perpendicular to the current, in Tesla.
    let b_val = lift::<FloatType>(2.0);
    let b_field = blade(idx_field_plane, b_val, metric);
    print_inputs(j_val, b_val);

    // 4. The physics, in one line that holds for any geometry: F = J . B.
    let force = j_vec.inner_product(&b_field);
    let force_val = *force
        .get(idx_force_direction)
        .expect("the force direction is a blade of this algebra");

    // 5. The sign is the reactor safety check. Euclidean (+1) gives the standard
    //    cross-product direction; Minkowski (-1) reverses it through the spacetime signature.
    print_force(force_val, poloidal_axis);
}

/// A multivector carrying `value` on a single blade of `metric`'s algebra.
fn blade(index: usize, value: FloatType, metric: Metric) -> CausalMultiVector<FloatType> {
    let mut data = vec![lift::<FloatType>(0.0); 1 << metric.dimension()];
    data[index] = value;
    CausalMultiVector::new(data, metric).expect("2^dim coefficients for this metric")
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("--- PLASMA FUSION SIMULATION: Reactor Confinement Check ---");
    println!("Context: A Stationary Tokamak Reactor (e.g., ITER).");
    println!(
        "Goal: Calculate the Lorentz Force vector to ensure Plasma is confined away from the walls."
    );
    println!(
        "Problem: Relativistic effects in high-energy plasma can alter geometric interactions."
    );
    println!("Solution: Use Geometric Algebra to automatically handle the spacetime signature.\n");
}

fn print_scenario(label: &str, description: &str) {
    println!(">> SCENARIO {label}: {description}");
}

fn print_separator() {
    println!("\n------------------------------------------------------------\n");
}

fn print_geometry(toroidal_axis: usize, poloidal_axis: usize) {
    println!(
        "  [Geometry] Current Axis: e_{toroidal_axis}, Field Plane: e_{toroidal_axis}e_{poloidal_axis}"
    );
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_inputs(j: FloatType, b: FloatType) {
    println!("  [Input] Plasma Current J: {:.1}", lower(j));
    println!("  [Input] Magnetic Field B: {:.1}", lower(b));
}

fn print_force(force: FloatType, poloidal_axis: usize) {
    println!(
        "  [Output] Lorentz Force F: {:.2} e_{}",
        lower(force),
        poloidal_axis
    );
    if lower(force) > 0.0 {
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
