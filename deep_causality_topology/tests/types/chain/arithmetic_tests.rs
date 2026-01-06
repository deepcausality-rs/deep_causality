/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::{Chain, Simplex, SimplicialComplex, Skeleton};
use std::sync::Arc;

fn create_test_complex() -> Arc<SimplicialComplex<f64>> {
    let vertices = vec![Simplex::new(vec![0]), Simplex::new(vec![1])];
    let skeleton_0 = Skeleton::new(0, vertices);

    // Just a dummy setup
    let skeletons = vec![skeleton_0];
    let boundary_ops = vec![CsrMatrix::new()];
    let coboundary_ops = vec![CsrMatrix::new()];

    Arc::new(SimplicialComplex::new(
        skeletons,
        boundary_ops,
        coboundary_ops,
        Vec::new(),
    ))
}

#[test]
fn test_chain_arithmetic_owned() {
    let complex = create_test_complex();

    let w1 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 10.0)]).unwrap();
    let c1 = Chain::new(Arc::clone(&complex), 0, w1);

    let w2 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 5.0)]).unwrap();
    let c2 = Chain::new(Arc::clone(&complex), 0, w2);

    // Test owned Add
    let sum = c1 + c2;
    assert_eq!(sum.weights().values(), &vec![15.0]);

    // Recreate consumed chains
    let w3 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 10.0)]).unwrap();
    let c3 = Chain::new(Arc::clone(&complex), 0, w3);

    let w4 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 2.0)]).unwrap();
    let c4 = Chain::new(Arc::clone(&complex), 0, w4);

    // Test owned Sub
    let diff = c3 - c4;
    assert_eq!(diff.weights().values(), &vec![8.0]);

    // Recreate
    let w5 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 5.0)]).unwrap();
    let c5 = Chain::new(Arc::clone(&complex), 0, w5);

    // Test owned Mul (Scalar)
    let scaled = c5 * 2.0;
    assert_eq!(scaled.weights().values(), &vec![10.0]);

    // Recreate
    let w6 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 5.0)]).unwrap();
    let c6 = Chain::new(Arc::clone(&complex), 0, w6);

    // Test owned Neg
    let neg = -c6;
    assert_eq!(neg.weights().values(), &vec![-5.0]);
}
