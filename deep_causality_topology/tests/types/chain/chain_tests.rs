/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::Chain;
use deep_causality_topology::utils_tests::create_triangle_complex;
use std::sync::Arc;

#[test]
fn test_chain_new() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 1; // Edges
    let weights = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0), (0, 2, 0.5)]).unwrap();

    let chain = Chain::new(complex.clone(), grade, weights.clone());

    assert_eq!(chain.complex(), &complex);
    assert_eq!(chain.grade(), grade);
    assert_eq!(chain.weights(), &weights);
}

#[test]
fn test_chain_getters() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 0; // Vertices
    let weights = CsrMatrix::from_triplets(1, 3, &[(0, 1, 1.0)]).unwrap(); // Vertex 1 with weight 1.0

    let chain = Chain::new(complex.clone(), grade, weights.clone());

    assert_eq!(chain.complex(), &complex);
    assert_eq!(chain.grade(), grade);
    assert_eq!(chain.weights(), &weights);
}

#[test]
fn test_chain_display() {
    let complex = Arc::new(create_triangle_complex());
    let grade = 1;
    let weights = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0), (0, 2, 0.5)]).unwrap();
    let chain = Chain::new(complex, grade, weights.clone());

    let expected = format!("Chain:\n  Grade: 1\n  Weights: {:?}\n", weights);
    assert_eq!(format!("{}", chain), expected);
}
