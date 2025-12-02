/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hodge Theory Example: Detecting Holes ===\n");

    // ------------------------------------------------------------------------
    // ENGINEERING VALUE:
    // Hodge Theory provides a powerful connection between differential geometry and
    // algebraic topology. The Hodge-Laplacian operator (Δ) is central to this,
    // as its kernel (solutions to Δω = 0) corresponds to the "harmonic forms"
    // on a manifold.
    //
    // The number of independent harmonic k-forms is equal to the k-th Betti number (b_k),
    // which counts the number of k-dimensional holes.
    //
    // This example demonstrates:
    // 1. Constructing a manifold with a 1D hole (a discrete annulus/cylinder).
    // 2. Defining a 1-form (a vector field on edges) that flows around the hole.
    // 3. Computing its Laplacian, Δω.
    // 4. Showing that Δω is nearly zero, proving the form is harmonic and thus
    //    confirming the presence of a non-trivial 1D hole (b₁ > 0).
    // ------------------------------------------------------------------------

    // 1. Create a manifold with a hole (a ring of 6 triangles)
    let points = CausalTensor::new(
        vec![
            1.0, 0.0, // v0
            0.5, 0.866, // v1
            -0.5, 0.866, // v2
            -1.0, 0.0, // v3
            -0.5, -0.866, // v4
            0.5, -0.866, // v5
        ],
        vec![6, 2],
    )?;
    let point_cloud = PointCloud::new(points, CausalTensor::new(vec![0.0; 6], vec![6])?, 0)?;
    let complex = point_cloud.triangulate(1.1)?;
    let num_simplices = complex
        .skeletons()
        .iter()
        .map(|s| s.simplices().len())
        .sum();

    println!(
        "Created a manifold with a hole ({} vertices, {} edges, {} faces).\n",
        complex.skeletons()[0].simplices().len(),
        complex.skeletons()[1].simplices().len(),
        complex
            .skeletons()
            .get(2)
            .map_or(0, |s| s.simplices().len())
    );

    // 2. Define a 1-form that flows around the hole.
    // We set a value of 1.0 on the "outer" edges of the ring.
    // Vertices: 6, Edges: 6, Faces: 0 in this triangulation
    let mut form_data = vec![0.0; num_simplices];
    // Assign 1.0 to the 1-forms (edges). Indices 6 to 11 are edges.
    for item in form_data.iter_mut().skip(6).take(6) {
        *item = 1.0;
    }
    let tensor = CausalTensor::new(form_data, vec![num_simplices])?;
    let manifold = Manifold::new(complex, tensor, 0)?;

    println!("\nDefined a 1-form ω that flows around the ring.");

    // 3. Compute the Laplacian of the 1-form: Δω = dδω + δdω
    println!("Computing Laplacian Δω...");
    let laplacian_form = manifold.laplacian(1); // Laplacian on 1-forms

    // 4. Check if the form is harmonic (Δω ≈ 0)
    let is_harmonic = laplacian_form.as_slice().iter().all(|&v| v.abs() < 1e-9);

    println!("\nResult of Δω: {:?}", laplacian_form.as_slice());

    if is_harmonic {
        println!("\nSUCCESS: The 1-form is harmonic (Δω ≈ 0).");
        println!("This proves the existence of a 1-dimensional hole in the manifold.");
        println!("The first Betti number, b₁, is greater than zero.");
    } else {
        println!("\nFAILURE: The 1-form is not harmonic.");
    }

    assert!(is_harmonic);

    Ok(())
}
