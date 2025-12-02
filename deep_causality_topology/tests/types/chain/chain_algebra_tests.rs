/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::{Chain, Simplex, SimplicialComplex, Skeleton};
use std::sync::Arc;

/// Helper to create a minimal simplicial complex for testing
fn create_test_complex() -> Arc<SimplicialComplex> {
    // Create a simple triangle complex:
    // Vertices: 0, 1, 2
    // Edges: (0,1), (0,2), (1,2)
    // Triangle: (0,1,2)

    // Skeleton 0: 3 vertices
    let vertices = vec![
        Simplex::new(vec![0]),
        Simplex::new(vec![1]),
        Simplex::new(vec![2]),
    ];
    let skeleton_0 = Skeleton::new(0, vertices);

    // Skeleton 1: 3 edges
    let edges = vec![
        Simplex::new(vec![0, 1]),
        Simplex::new(vec![0, 2]),
        Simplex::new(vec![1, 2]),
    ];
    let skeleton_1 = Skeleton::new(1, edges);

    // Skeleton 2: 1 triangle
    let triangles = vec![Simplex::new(vec![0, 1, 2])];
    let skeleton_2 = Skeleton::new(2, triangles);

    let skeletons = vec![skeleton_0, skeleton_1, skeleton_2];

    // Boundary operators (we can use empty matrices for algebra tests)
    let boundary_ops = vec![
        CsrMatrix::new(), // ∂_0
        CsrMatrix::new(), // ∂_1
        CsrMatrix::new(), // ∂_2
    ];

    let coboundary_ops = vec![
        CsrMatrix::new(), // ∂*_0
        CsrMatrix::new(), // ∂*_1
        CsrMatrix::new(), // ∂*_2
    ];

    Arc::new(SimplicialComplex::new(
        skeletons,
        boundary_ops,
        coboundary_ops,
    ))
}

#[test]
fn test_chain_zero() {
    let complex = create_test_complex();

    // Create a zero chain on grade 1 (edges)
    let zero_chain = Chain::<f64>::zero(Arc::clone(&complex), 1);

    // Verify it has the right shape (1 x 3 for 3 edges)
    assert_eq!(zero_chain.weights().shape(), (1, 3));
    assert!(zero_chain.weights().values().is_empty()); // All zeros
}

#[test]
fn test_chain_addition() {
    let complex = create_test_complex();

    // Create two chains with different weights on edges
    let w1 = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0), (0, 1, 2.0)]).unwrap();
    let c1 = Chain::new(Arc::clone(&complex), 1, w1);

    let w2 = CsrMatrix::from_triplets(1, 3, &[(0, 1, 3.0), (0, 2, 4.0)]).unwrap();
    let c2 = Chain::new(Arc::clone(&complex), 1, w2);

    // Test addition using the + operator
    let sum = &c1 + &c2;

    // Expected: [1.0, 5.0, 4.0]
    assert_eq!(sum.weights().values(), &vec![1.0, 5.0, 4.0]);
    assert_eq!(sum.grade(), 1);
}

#[test]
fn test_chain_subtraction() {
    let complex = create_test_complex();

    let w1 = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0), (0, 1, 5.0)]).unwrap();
    let c1 = Chain::new(Arc::clone(&complex), 1, w1);

    let w2 = CsrMatrix::from_triplets(1, 3, &[(0, 1, 3.0), (0, 2, 4.0)]).unwrap();
    let c2 = Chain::new(Arc::clone(&complex), 1, w2);

    // Test subtraction using the - operator
    let diff = &c1 - &c2;

    // Expected: [1.0, 2.0, -4.0]
    assert_eq!(diff.weights().values(), &vec![1.0, 2.0, -4.0]);
}

#[test]
fn test_chain_negation() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0), (0, 1, 2.0)]).unwrap();
    let c = Chain::new(Arc::clone(&complex), 1, w);

    // Test negation using the unary - operator
    let neg = -&c;

    // Expected: [-1.0, -2.0]
    assert_eq!(neg.weights().values(), &vec![-1.0, -2.0]);
}

#[test]
fn test_chain_scalar_multiplication() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0), (0, 1, 2.0)]).unwrap();
    let c = Chain::new(Arc::clone(&complex), 1, w);

    // Test scalar multiplication using the * operator
    let scaled = &c * 3.0;

    // Expected: [3.0, 6.0]
    assert_eq!(scaled.weights().values(), &vec![3.0, 6.0]);
}

#[test]
fn test_chain_add_with_zero() {
    let complex = create_test_complex();

    // Use 1.0 as "zero" for contextual sparsity
    let w1 = CsrMatrix::from_triplets_with_zero(1, 3, &[(0, 0, 1.0), (0, 1, 2.0)], 1.0).unwrap();
    let c1 = Chain::new(Arc::clone(&complex), 1, w1);

    let w2 = CsrMatrix::from_triplets_with_zero(1, 3, &[(0, 1, 1.0)], 1.0).unwrap();
    let c2 = Chain::new(Arc::clone(&complex), 1, w2);

    // Add with explicit zero value
    let sum = c1.add_with_zero(&c2, 1.0);

    // c1 has value 2.0 at index 1 (index 0 was filtered as "zero")
    // c2 has nothing (index 1 was filtered as "zero")
    // sum should have 2.0 at index 1
    assert_eq!(sum.weights().values(), &vec![2.0]);
}

#[test]
#[should_panic(expected = "Chain grade mismatch")]
fn test_chain_incompatible_grade() {
    let complex = create_test_complex();

    let w1 = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0)]).unwrap();
    let c1 = Chain::new(Arc::clone(&complex), 1, w1); // grade 1

    let w2 = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).unwrap();
    let c2 = Chain::new(Arc::clone(&complex), 2, w2); // grade 2

    // This should panic
    let _ = &c1 + &c2;
}

#[test]
#[should_panic(expected = "Chain complex mismatch")]
fn test_chain_incompatible_complex() {
    let complex1 = create_test_complex();
    let complex2 = create_test_complex(); // Different Arc, different complex

    let w1 = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0)]).unwrap();
    let c1 = Chain::new(complex1, 1, w1);

    let w2 = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0)]).unwrap();
    let c2 = Chain::new(complex2, 1, w2);

    // This should panic
    let _ = &c1 + &c2;
}
