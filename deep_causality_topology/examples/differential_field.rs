/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Differential Field Example: Heat Equation ===\n");

    // ------------------------------------------------------------------------
    // ENGINEERING VALUE:
    // Many physical processes (diffusion, fluid flow, electromagnetism) are described
    // by partial differential equations (PDEs). When data lives on a complex,
    // non-grid-like structure (like a sensor network or a 3D model), we need
    // tools from differential geometry to solve these equations.
    //
    // This example demonstrates how to solve the Heat Equation (∂u/∂t = -Δu) on a
    // discrete manifold. The Laplacian operator (Δ) measures the local curvature
    // of a field and drives the diffusion process.
    //
    // This allows us to simulate how a quantity (like heat or information) spreads
    // across a complex topology.
    // ------------------------------------------------------------------------

    // 1. Setup (Triangle)
    let points = CausalTensor::new(
        vec![
            0.0, 0.0, // v0
            1.0, 0.0, // v1
            0.5, 0.866, // v2 (Equilateral)
        ],
        vec![3, 2],
    )?;
    let point_cloud = PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3])?, 0)?;
    let complex = point_cloud.triangulate(1.1)?;

    // 2. Initial State (Hot Vertex 0)
    let num_simplices = complex.total_simplices();
    let mut initial_data = vec![0.0; num_simplices];
    initial_data[0] = 100.0; // Heat at v0

    let mut manifold = Manifold::new(
        complex.clone(),
        CausalTensor::new(initial_data, vec![num_simplices])?,
        0,
    )?;

    // 3. Simulation
    // Using a smaller timestep for stability with the new metric
    let dt = 0.005;
    let steps = 50;

    println!("Starting Diffusion...");

    for i in 0..=steps {
        // L = d* delta
        let laplacian = manifold.laplacian(0);

        let current = manifold.data().as_slice();
        let delta = laplacian.as_slice();
        let mut next = current.to_vec();

        // Update Vertices (0-forms)
        // Heat Eq: du/dt = - L u
        let n_verts = complex.skeletons()[0].simplices().len();
        for v in 0..n_verts {
            // Mass Matrix Integration:
            // The Laplacian operator we computed includes the metric weights.
            // Standard update: u_new = u_old - dt * (L * u)
            // Note: If L is the weighted Laplacian, we might need to divide by Mass_0
            // if we haven't already.
            // In this implementation, codifferential includes M^{-1}, so result is pointwise derivative.
            next[v] -= dt * delta[v];
        }

        manifold = Manifold::new(
            complex.clone(),
            CausalTensor::new(next, vec![num_simplices])?,
            0,
        )?;

        if i % 10 == 0 {
            let v_data = &manifold.data().as_slice()[0..3];
            println!(
                "Step {:2}: [{:.2}, {:.2}, {:.2}]",
                i, v_data[0], v_data[1], v_data[2]
            );
        }
    }

    // Verification: Total Energy Conservation?
    // In a closed system, heat spreads but sum(u_i * Mass_i) should be constant.
    // Or simply, temperature equilibrates.
    let final_v: &[f64] = &manifold.data().as_slice()[0..3];
    println!(
        "Final:   [{:.2}, {:.2}, {:.2}]",
        final_v[0], final_v[1], final_v[2]
    );

    if (final_v[0] - final_v[1]).abs() < 1.0 && (final_v[0] - final_v[2]).abs() < 1.0 {
        println!(">> SUCCESS: Heat diffused to equilibrium.");
    } else {
        println!(">> WARNING: Non-equilibrium state.");
    }

    Ok(())
}
