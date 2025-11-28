/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{BoundedComonad, Functor};
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, ManifoldWitness, Simplex, SimplicialComplex, Skeleton};

// Helper to create a valid manifold (line segment)
fn create_line_manifold() -> Manifold<f64> {
    let vertices = vec![Simplex::new(vec![0]), Simplex::new(vec![1])];
    let skeleton_0 = Skeleton::new(0, vertices);

    let edges = vec![Simplex::new(vec![0, 1])];
    let skeleton_1 = Skeleton::new(1, edges);

    let d1 = CsrMatrix::from_triplets(2, 1, &[(1, 0, 1i8), (0, 0, -1)]).unwrap();

    let complex = SimplicialComplex::new(vec![skeleton_0, skeleton_1], vec![d1], vec![]);

    // Data on 0-simplices (vertices) and 1-simplices (edges)
    // Total simplices = 2 + 1 = 3
    // Order: vertices first, then edges (based on skeleton order in complex)
    // Actually Manifold stores data in a single tensor.
    // The mapping depends on how Manifold uses the data.
    // Usually it's 1-to-1 with the total number of simplices if it's a general field,
    // OR it might be specific to the max dimension.
    // Looking at Manifold struct, it has `data: CausalTensor<T>`.
    // Let's assume it maps to all simplices for this test.
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();

    Manifold::new(complex, data, 0).expect("Failed to create manifold")
}

#[test]
fn test_manifold_functor() {
    let manifold = create_line_manifold();
    // Data: [1.0, 2.0, 3.0]

    let mapped = ManifoldWitness::fmap(manifold, |x| x * 2.0);

    assert_eq!(mapped.data().as_slice(), &[2.0, 4.0, 6.0]);
}

#[test]
fn test_manifold_extract() {
    // Move cursor to 1 (second vertex)
    // Manifold struct has public crate fields, but we should use a method if available or just rely on HKT which uses internal access?
    // HKT implementation uses `fa.cursor`.
    // We can't set cursor easily on the struct since fields are private to crate.
    // But `Manifold::new` sets cursor to 0.
    // Wait, `Manifold::new` takes `cursor` as argument.

    // Re-create with cursor 1
    let complex = create_line_manifold().complex().clone(); // Clone the complex from helper
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();
    let manifold = Manifold::new(complex, data, 1).unwrap();

    let val = ManifoldWitness::extract(&manifold);
    assert_eq!(val, 20.0);
}

#[test]
fn test_manifold_extend() {
    let manifold = create_line_manifold();
    // Data: [1.0, 2.0, 3.0] (Vertices: 0, 1; Edge: 0-1)

    // Extend: Value + Cursor Index
    let extended = ManifoldWitness::extend(&manifold, |w| {
        let val = ManifoldWitness::extract(w);
        val + (w.cursor() as f64)
    });

    // Index 0: 1.0 + 0.0 = 1.0
    // Index 1: 2.0 + 1.0 = 3.0
    // Index 2: 3.0 + 2.0 = 5.0
    assert_eq!(extended.data().as_slice(), &[1.0, 3.0, 5.0]);
}
