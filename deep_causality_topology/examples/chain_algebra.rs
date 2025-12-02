/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::{Chain, Simplex, SimplicialComplex, Skeleton};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Chain Algebra Example: ∂∂ = 0 ===\n");

    // ------------------------------------------------------------------------
    // ENGINEERING VALUE:
    // A fundamental theorem in algebraic topology is that "the boundary of a boundary is zero" (∂∂=0).
    // This property is the foundation of homology theory, which allows us to count "holes"
    // in data and is crucial for robust topological data analysis.
    //
    // This example demonstrates:
    // 1. Constructing a simplicial complex (a single tetrahedron).
    // 2. Creating a 2-chain (a weighted sum of triangles).
    // 3. Computing the first boundary (a 1-chain of edges).
    // 4. Computing the second boundary and verifying it is the zero chain.
    // ------------------------------------------------------------------------

    // 1. Define a simplicial complex: a single tetrahedron
    //    Vertices: 0, 1, 2, 3
    //    Edges: (0,1), (0,2), (0,3), (1,2), (1,3), (2,3)
    //    Faces: (0,1,2), (0,1,3), (0,2,3), (1,2,3)
    //    Volume: (0,1,2,3)
    let vertices = vec![
        Simplex::new(vec![0]),
        Simplex::new(vec![1]),
        Simplex::new(vec![2]),
        Simplex::new(vec![3]),
    ];
    let skeleton_0 = Skeleton::new(0, vertices);

    let edges = vec![
        Simplex::new(vec![0, 1]), // 0
        Simplex::new(vec![0, 2]), // 1
        Simplex::new(vec![0, 3]), // 2
        Simplex::new(vec![1, 2]), // 3
        Simplex::new(vec![1, 3]), // 4
        Simplex::new(vec![2, 3]), // 5
    ];
    let skeleton_1 = Skeleton::new(1, edges);

    let faces = vec![
        Simplex::new(vec![0, 1, 2]), // 0
        Simplex::new(vec![0, 1, 3]), // 1
        Simplex::new(vec![0, 2, 3]), // 2
        Simplex::new(vec![1, 2, 3]), // 3
    ];
    let skeleton_2 = Skeleton::new(2, faces);

    // Boundary operator d1: Edges -> Vertices
    let d1 = CsrMatrix::from_triplets(
        4,
        6,
        &[
            (1, 0, 1),
            (0, 0, -1), // d(e0)=v1-v0
            (2, 1, 1),
            (0, 1, -1), // d(e1)=v2-v0
            (3, 2, 1),
            (0, 2, -1), // d(e2)=v3-v0
            (2, 3, 1),
            (1, 3, -1), // d(e3)=v2-v1
            (3, 4, 1),
            (1, 4, -1), // d(e4)=v3-v1
            (3, 5, 1),
            (2, 5, -1), // d(e5)=v3-v2
        ],
    )?;

    // Boundary operator d2: Faces -> Edges
    let d2 = CsrMatrix::from_triplets(
        6,
        4,
        &[
            (3, 0, 1),
            (1, 0, -1),
            (0, 0, 1), // d(f0)=e3-e1+e0
            (4, 1, 1),
            (2, 1, -1),
            (0, 1, 1), // d(f1)=e4-e2+e0
            (5, 2, 1),
            (2, 2, -1),
            (1, 2, 1), // d(f2)=e5-e2+e1
            (5, 3, 1),
            (4, 3, -1),
            (3, 3, 1), // d(f3)=e5-e4+e3
        ],
    )?;

    let complex = Arc::new(SimplicialComplex::new(
        vec![skeleton_0, skeleton_1, skeleton_2],
        vec![d1, d2],
        vec![], // Coboundaries not needed for this example
        vec![], // Hodge star not needed
    ));

    println!(
        "Created a tetrahedral complex with {} vertices, {} edges, and {} faces.",
        complex.skeletons()[0].simplices().len(),
        complex.skeletons()[1].simplices().len(),
        complex.skeletons()[2].simplices().len()
    );

    // 2. Create a 2-chain: c = 1.0 * f0 + 2.0 * f1
    // where f0 is face (0,1,2) and f1 is face (0,1,3)
    let weights_c = CsrMatrix::from_triplets(1, 4, &[(0, 0, 1.0), (0, 1, 2.0)])?;
    let chain_c = Chain::new(complex.clone(), 2, weights_c);
    println!("\nInitial 2-chain (c):\n{}", chain_c);

    // 3. Compute the first boundary: b = ∂c
    let chain_b = complex.boundary(&chain_c);
    println!("First boundary ∂c (a 1-chain of edges):\n{}", chain_b);
    assert_eq!(chain_b.grade(), 1);
    assert!(
        !chain_b.weights().values().is_empty(),
        "First boundary should not be zero"
    );

    // 4. Compute the second boundary: z = ∂b = ∂(∂c)
    let chain_z: Chain<f32> = complex.boundary(&chain_b);
    println!(
        "Second boundary ∂(∂c) (a 0-chain of vertices):\n{}",
        chain_z
    );
    assert_eq!(chain_z.grade(), 0);

    // 5. Verify that the second boundary is zero
    let is_zero = chain_z.weights().values().iter().all(|&w| w.abs() < 1e-9);
    assert!(is_zero, "The boundary of a boundary (∂∂c) must be zero.");

    println!("\nVerification successful: ∂∂c = 0");

    Ok(())
}
