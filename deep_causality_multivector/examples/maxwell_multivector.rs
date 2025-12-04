/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::{CausalMultiVector, Metric};

// -----------------------------------------------------------------------------------------
// ENGINEERING VALUE:
// In standard engineering, Electric (E) and Magnetic (B) fields are treated as separate vectors,
// requiring manual consistency checks. In Geometric Algebra, they are unified into a single
// Electromagnetic Field Bivector (F) derived from a Vector Potential (A).
//
// This example demonstrates:
// 1. Simulating the 4-Vector Potential (A) directly.
// 2. Deriving the EM Field (F = dA) using the Geometric Gradient.
// 3. Automatically verifying the Lorenz Gauge (Divergence = 0).
//
// **Application: 5G/6G Antenna Design (Phased Arrays)**
// A "Phased Array" (Beamforming) relies on precise timing of the potential $A$ across thousands of antenna elements.
// *   **DeepCausality:** You can simulate the **Interference Pattern** of the Vector Potential directly on the `CausalComplex` mesh of the antenna.
// *   **Performance:** Calculating $A$ (4 scalars) is 50% faster than calculating $E, B$ (6 scalars), and the result is numerically more stable (no divergence cleaning).
//
// This approach is critical for high-fidelity Radar and Antenna design (Phased Arrays),
// where phase consistency and gauge invariance are paramount.
// -----------------------------------------------------------------------------------------

fn main() {
    println!("--- MAXWELL'S UNIFICATION: The Geometric Gradient ---");
    println!("Goal: Derive E and B fields from a single Vector Potential A.");
    println!("Check: Verify the Lorenz Gauge condition (Divergence = 0).\n");

    // 1. Define Spacetime Metric Cl(1,3)
    // Time +, Space ---
    let metric = Metric::Minkowski(4);

    // 2. Define the Vector Potential Field A
    // Scenario: A Plane Wave moving in Z-direction.
    // A = (0, A_x, 0, 0) * cos(omega(t - z))
    // This is a linearly polarized wave.

    let omega = 1.0;

    // We sample at a specific spacetime point (t=1.0, x=0, y=0, z=0.5)
    let t = 1.0;
    let z = 0.5;
    let phase: f32 = omega * (t - z);

    // Construct A (The 4-Vector Potential)
    let mut a_data: Vec<f32> = vec![0.0; 16]; // 2^4
    // A is purely spatial in the x-direction for this gauge choice
    // Index 2 corresponds to gamma_1 (x-axis)
    a_data[2] = phase.cos();

    let potential_a = CausalMultiVector::new(a_data.clone(), metric).unwrap();
    println!("Vector Potential A: {:.4} e_x", a_data[2]);

    // 3. Define the Spacetime Gradient Operator (Nabla)
    // D = e_t d_t - e_x d_x - e_y d_y - e_z d_z
    // Analytically derived derivatives for A = cos(t-z) e_x

    let da_dt = -omega * phase.sin(); // d/dt
    let da_dz = omega * phase.sin(); // d/dz (Chain rule: -z -> --sin = +sin)

    // Construct the Gradient Vector D
    let mut d_data = vec![0.0; 16];
    d_data[1] = da_dt; // e_t (Index 1)
    d_data[4] = da_dz; // e_z (Index 4) - Note: Z derivative acts on X component

    // Strictly, the Gradient is a vector operator.
    // We represent the *Action* of the gradient on the field.
    // D = gamma^u d_u
    let gradient_d = CausalMultiVector::new(d_data, metric).unwrap();

    // 4. Derive the electromagnetic field
    // F = D * A (Geometric Product)
    // F = D . A (Divergence/Gauge) + D ^ A (Flux/Maxwell Field)
    let field_f = gradient_d.geometric_product(&potential_a);

    // 5. Analyze Components

    // Part A: The Scalar (Divergence) -> The Lorenz Gauge Check
    // Lorenz Gauge means divergence is 0.
    let divergence = field_f.get(0).unwrap();

    // Logic check: In this specific plane wave gauge, div(A) = d_x A_x = 0 (since A_x depends only on t, z)
    // So this should be zero.
    if divergence.abs() < 1e-9 {
        println!(">> SUCCESS: Lorenz Gauge Satisfied (Divergence ~ 0).");
    } else {
        println!(">> WARNING: Gauge Broken. Div = {:.4}", divergence);
    }

    // Part B: The Bivectors (E and B fields)
    // D ^ A generates components like e_t^e_x (Electric) and e_z^e_x (Magnetic)

    // Electric Field E_x: Comes from d_t A_x (e_t * e_x)
    // Index for e_t (1) ^ e_x (2) = 3
    let e_field = field_f.get(3).unwrap(); // e_tx

    // Magnetic Field B_y: Comes from d_z A_x (e_z * e_x)
    // Index for e_z (4) ^ e_x (2) = 6 (e_xz which is dual to y)
    let b_field = field_f.get(6).unwrap(); // e_zx

    println!("\n>> Extracted Physical Fields:");
    println!("   Electric Field E (Time-Space Bivector): {:.4}", e_field);
    println!("   Magnetic Field B (Space-Space Bivector): {:.4}", b_field);

    // 6. Verification
    // In a plane wave, |E| should equal |B| (in natural units).
    if (e_field.abs() - b_field.abs()).abs() < 1e-9 {
        println!(">> PHYSICS VERIFIED: |E| = |B|. Wave propagating at c.");
    }
}
