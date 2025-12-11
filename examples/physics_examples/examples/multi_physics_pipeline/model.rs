/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, Simplex, SimplicialComplex, SimplicialComplexBuilder};

pub(crate) fn make_1d_manifold(data: Vec<f64>) -> Manifold<f64> {
    let n = data.len(); // 10
    let mut builder = SimplicialComplexBuilder::new(1);

    // Add edges (this adds vertices automatically)
    // Edges 0..(n-1)
    for i in 0..n - 1 {
        // Simplex::new sorts vertices.
        builder
            .add_simplex(Simplex::new(vec![i, i + 1]))
            .expect("Failed to add simplex");
    }
    // Need to ensure last vertex is added if not covered?
    // Edge (n-2, n-1) covers n-1.
    // Vertices are 0..n. Edges are 0..n-1.
    // If n=10, vertices 0..9.
    // Last edge (8, 9). Covers 9. All good.
    let complex = builder.build().expect("Failed to build complex");

    // Extract computed operators
    let skeletons = complex.skeletons().clone();
    let boundaries = complex.boundary_operators().clone();
    let coboundaries = complex.coboundary_operators().clone();

    // Manual Hodge Star / Mass Matrix Construction
    // Code in codifferential expects indices aligned with k-simplices, meaning square Mass Matrices.
    // M0: 10x10 Identity.
    // M1: 9x9 Identity.
    let n0 = skeletons[0].simplices().len(); // 10
    let n1 = skeletons[1].simplices().len(); // 9

    // Mass 0 (10x10)
    let mut triplets0 = Vec::new();
    for i in 0..n0 {
        triplets0.push((i, i, 1.0));
    }
    let h0 = CsrMatrix::from_triplets(n0, n0, &triplets0).unwrap();

    // Mass 1 (9x9)
    let mut triplets1 = Vec::new();
    for i in 0..n1 {
        triplets1.push((i, i, 1.0));
    }
    let h1 = CsrMatrix::from_triplets(n1, n1, &triplets1).unwrap();

    let hodge = vec![h0, h1];

    // Reconstruct complex with hodge
    let complex_with_hodge = SimplicialComplex::new(skeletons, boundaries, coboundaries, hodge);

    // Data Tensor needs size = total simplices.
    // Builder adds simplices.
    // n0 = 10, n1 = 9. Total = 19.
    let mut full_data = data;
    full_data.resize(n0 + n1, 0.0);

    let len = full_data.len();
    let tensor = CausalTensor::new(full_data, vec![len]).unwrap();

    Manifold::new(complex_with_hodge, tensor, 0).expect("Failed to create valid manifold")
}
