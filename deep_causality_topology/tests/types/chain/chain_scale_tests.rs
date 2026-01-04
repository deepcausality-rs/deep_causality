/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::{Chain, Simplex, SimplicialComplex, Skeleton};
use std::sync::Arc;

fn create_test_complex() -> Arc<SimplicialComplex> {
    let vertices = vec![
        Simplex::new(vec![0]),
        Simplex::new(vec![1]),
        Simplex::new(vec![2]),
    ];
    let skeleton_0 = Skeleton::new(0, vertices);

    let edges = vec![
        Simplex::new(vec![0, 1]),
        Simplex::new(vec![0, 2]),
        Simplex::new(vec![1, 2]),
    ];
    let skeleton_1 = Skeleton::new(1, edges);

    let skeletons = vec![skeleton_0, skeleton_1];
    let boundary_ops = vec![CsrMatrix::new(), CsrMatrix::new()];
    let coboundary_ops = vec![CsrMatrix::new(), CsrMatrix::new()];

    Arc::new(SimplicialComplex::new(
        skeletons,
        boundary_ops,
        coboundary_ops,
        Vec::new(),
    ))
}

// ============================================================================
// Chain::scale() tests
// ============================================================================

#[test]
fn test_chain_scale_by_positive_scalar() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 2.0), (0, 1, 3.0)]).unwrap();
    let chain = Chain::new(Arc::clone(&complex), 1, w);

    let scaled = chain.scale(2.0);

    assert_eq!(scaled.weights().values(), &vec![4.0, 6.0]);
    assert_eq!(scaled.grade(), 1);
}

#[test]
fn test_chain_scale_by_negative_scalar() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 2.0), (0, 2, 4.0)]).unwrap();
    let chain = Chain::new(Arc::clone(&complex), 1, w);

    let scaled = chain.scale(-1.0);

    assert_eq!(scaled.weights().values(), &vec![-2.0, -4.0]);
}

#[test]
fn test_chain_scale_by_zero() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 5.0), (0, 1, 10.0)]).unwrap();
    let chain = Chain::new(Arc::clone(&complex), 1, w);

    let scaled = chain.scale(0.0);

    // All values become 0, sparse matrix should be empty
    assert!(
        scaled.weights().values().is_empty() || scaled.weights().values().iter().all(|&v| v == 0.0)
    );
}

#[test]
fn test_chain_scale_by_fractional_scalar() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 10.0), (0, 1, 20.0)]).unwrap();
    let chain = Chain::new(Arc::clone(&complex), 1, w);

    let scaled = chain.scale(0.5);

    assert_eq!(scaled.weights().values(), &vec![5.0, 10.0]);
}

#[test]
fn test_chain_scale_preserves_complex_reference() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0)]).unwrap();
    let chain = Chain::new(Arc::clone(&complex), 1, w);

    let scaled = chain.scale(3.0);

    // Verify the complex reference is preserved
    assert!(Arc::ptr_eq(scaled.complex(), &complex));
}

#[test]
fn test_chain_scale_preserves_grade() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0)]).unwrap();
    let chain = Chain::new(Arc::clone(&complex), 0, w); // Grade 0 (vertices)

    let scaled = chain.scale(5.0);

    assert_eq!(scaled.grade(), 0);
}

// ============================================================================
// Operator trait tests for scalar multiplication
// ============================================================================

#[test]
fn test_chain_mul_operator_owned() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 2.0), (0, 1, 3.0)]).unwrap();
    let chain = Chain::new(Arc::clone(&complex), 1, w);

    // Test owned * scalar using Mul trait
    let result = chain * 2.0;

    assert_eq!(result.weights().values(), &vec![4.0, 6.0]);
}

#[test]
fn test_chain_mul_operator_ref() {
    let complex = create_test_complex();

    let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, 2.0), (0, 1, 3.0)]).unwrap();
    let chain = Chain::new(Arc::clone(&complex), 1, w);

    // Test &Chain * scalar using Mul trait
    let result = &chain * 2.0;

    assert_eq!(result.weights().values(), &vec![4.0, 6.0]);
    // Original chain should still be accessible
    assert_eq!(chain.grade(), 1);
}
