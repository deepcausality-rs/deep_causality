/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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

    // 1. Create a triangulated manifold (a single triangle)
    //
    //  (0)
    //  / \
    // (1)-(2)
    let points = CausalTensor::new(
        vec![
            0.0, 0.0, // v0
            1.0, 0.0, // v1
            0.5, 0.866, // v2
        ],
        vec![3, 2],
    )?;
    let point_cloud = PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3])?, 0)?;

    // Triangulate with a radius that connects all vertices of the triangle
    let complex = point_cloud.triangulate(1.1)?; // Max edge length is 1.0
    let num_simplices = complex
        .skeletons()
        .iter()
        .map(|s| s.simplices().len())
        .sum();

    println!("num_simplices: {}", num_simplices);
    println!("0-simplices: {}", complex.skeletons()[0].simplices().len());
    println!("1-simplices: {}", complex.skeletons()[1].simplices().len());
    println!("2-simplices: {}", complex.skeletons()[2].simplices().len());

    // 2. Define a scalar field `f` (heat distribution) on the manifold.
    // We place a "hot spot" at vertex 0.
    let mut heat_values = vec![0.0; num_simplices];
    heat_values[0] = 100.0; // Heat = 100.0 at vertex 0
    let heat_tensor = CausalTensor::new(heat_values, vec![num_simplices])?;

    let mut manifold = Manifold::new(complex, heat_tensor, 0)?;
    println!(
        "Created a 2D manifold with {} vertices, data len: {}.",
        manifold.complex().skeletons()[0].simplices().len(),
        manifold.data().len()
    );

    // 3. Simulate the Heat Equation using explicit Euler time-stepping.
    //    f(t+dt) = f(t) - dt * Δf(t)
    let dt = 0.01; // Time step
    let num_steps = 50;

    println!("\n--- Simulating Heat Diffusion ---");
    println!("Initial State: {:?}", manifold.data().as_slice());

    for i in 0..=num_steps {
        // Compute the Laplacian of the current heat distribution.
        // Heat is a scalar field on vertices (0-forms), so we use k=0
        let laplacian = manifold.laplacian(0);

        // Update the heat values at each vertex.
        let current_data = manifold.data().as_slice();
        let laplacian_data = laplacian.as_slice();
        let mut new_data = current_data.to_vec();

        // We only update the 0-forms (vertex values)
        for v_idx in 0..manifold.complex().skeletons()[0].simplices().len() {
            new_data[v_idx] -= dt * laplacian_data[v_idx];
        }

        // Create a new manifold with the updated data for the next step.
        manifold = Manifold::new(
            manifold.complex().clone(),
            CausalTensor::new(new_data, vec![num_simplices])?,
            0,
        )?;

        if i % 10 == 0 {
            // Print vertex data only for clarity
            let vertex_data: Vec<_> = manifold.data().as_slice()[..3]
                .iter()
                .map(|&x| format!("{:.2}", x))
                .collect(); // Adjusted for 3 vertices
            println!("Step {: >2}: {:?}", i, vertex_data);
        }
    }

    println!("\nFinal State: {:?}", manifold.data().as_slice());
    println!("\nNotice how the initial heat at vertex 0 has diffused across the manifold.");

    Ok(())
}
