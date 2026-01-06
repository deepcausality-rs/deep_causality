/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    BaseTopology, Manifold, ManifoldTopology, Simplex, SimplicialComplex, Skeleton,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Manifold Analysis Example ===");

    // ------------------------------------------------------------------------
    // ENGINEERING VALUE:
    // Manifolds provide a rigorous geometric framework for understanding the
    // "shape" of data. In causal AI, ensuring that a data structure forms a
    // valid manifold is often a prerequisite for applying geometric deep learning
    // or differential geometry operators (like curvature or flow).
    //
    // This example demonstrates:
    // 1. Constructing a simplicial complex representing a 1-manifold.
    // 2. Validating topological invariants (Euler Characteristic).
    // 3. Ensuring the structure is mathematically sound for further analysis.
    // ------------------------------------------------------------------------

    // Construct a simple 1-manifold (Line Segment: 0 -> 1)
    // Vertices: {0}, {1}
    // Edge: {0, 1}

    // 1. Define Skeletons
    let v0 = Simplex::new(vec![0]);
    let v1 = Simplex::new(vec![1]);
    let skeleton_0 = Skeleton::new(0, vec![v0, v1]);

    let e01 = Simplex::new(vec![0, 1]);
    let skeleton_1 = Skeleton::new(1, vec![e01]);

    // 2. Define Boundary Operator d1 (Edges -> Vertices)
    // d({0, 1}) = {1} - {0}
    // Matrix: Rows=Vertices(2), Cols=Edges(1)
    // (0, 0) -> -1 (Vertex 0)
    // (1, 0) -> 1  (Vertex 1)
    let d1 = CsrMatrix::from_triplets(2, 1, &[(0, 0, -1i8), (1, 0, 1i8)])?;

    // 3. Create Simplicial Complex
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(vec![skeleton_0, skeleton_1], vec![d1], vec![], vec![]);

    // 4. Create Manifold
    // Data for 3 simplices (2 vertices + 1 edge)
    let data = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3])?;
    let manifold = Manifold::new(complex, data, 0)?;

    println!("Manifold Created: {}", manifold);
    println!("Dimension: {}", manifold.dimension());

    // 5. Calculate Euler Characteristic
    // Chi = V - E = 2 - 1 = 1
    let chi = manifold.euler_characteristic();
    println!("Euler Characteristic: {}", chi);
    assert_eq!(chi, 1);

    // 6. Check Manifold Properties
    // Note: `check_is_manifold` is called during construction, so if we are here, it is valid.
    // We can check orientation.
    println!("Is Oriented: {}", manifold.is_oriented());

    Ok(())
}
