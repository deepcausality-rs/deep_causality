/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::utils_tests::create_line_complex;
use deep_causality_topology::{Chain, Simplex, SimplicialComplex, Skeleton};
use std::sync::Arc;

#[test]
fn test_simplicial_complex_boundary_d1() {
    let complex = Arc::new(create_line_complex()); // 2 vertices, 1 edge (0,1)
    let weights = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).unwrap(); // 1-chain: 1 * (0,1)
    let chain = Chain::new(complex.clone(), 1, weights);

    // Boundary of (0,1) is (1) - (0)
    let boundary_chain = complex.boundary(&chain);
    dbg!(&boundary_chain);

    assert_eq!(boundary_chain.grade(), 0);
    // Expecting: -1.0 * (0) + 1.0 * (1)
    let expected_weights = CsrMatrix::from_triplets(1, 2, &[(0, 0, -1.0), (0, 1, 1.0)]).unwrap();
    assert_eq!(boundary_chain.weights(), &expected_weights);
}

#[test]
#[should_panic(expected = "Cannot take boundary of 0-chain")]
fn test_simplicial_complex_boundary_d0_panic() {
    let complex = Arc::new(create_line_complex());
    let weights = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1.0)]).unwrap(); // 0-chain: 1 * (0)
    let chain = Chain::new(complex.clone(), 0, weights);

    complex.boundary(&chain);
}

#[test]
fn test_simplicial_complex_coboundary_d0() {
    // For coboundary, we need to manually set up coboundary operators or ensure triangulate does.
    // For simplicity, let's create a minimal complex with a manually defined coboundary.
    let vertices = std::vec![Simplex::new(std::vec![0]), Simplex::new(std::vec![1])];
    let skeleton_0 = Skeleton::new(0, vertices);
    let edges = std::vec![Simplex::new(std::vec![0, 1])];
    let skeleton_1 = Skeleton::new(1, edges);

    let b1 = CsrMatrix::from_triplets(2, 1, &[(1, 0, 1i8), (0, 0, -1)]).unwrap(); // d1: (0,1) -> (1)-(0)
    let c0 = b1.transpose(); // C0 = B1^T: (0) -> -(0,1), (1) -> (0,1)

    let complex = Arc::new(SimplicialComplex::new(
        std::vec![skeleton_0, skeleton_1],
        std::vec![b1],
        std::vec![c0], // Only c0
        std::vec![],
    ));

    // 0-chain: 1.0 * (0)
    let weights = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1.0)]).unwrap();
    let chain = Chain::new(complex.clone(), 0, weights);

    // Coboundary of (0) should be -(0,1)
    let coboundary_chain = complex.coboundary(&chain);

    assert_eq!(coboundary_chain.grade(), 1);
    let expected_weights = CsrMatrix::from_triplets(1, 1, &[(0, 0, -1.0)]).unwrap();
    assert_eq!(coboundary_chain.weights(), &expected_weights);
}

#[test]
#[should_panic(expected = "Cannot take coboundary of max-dim chain")]
fn test_simplicial_complex_coboundary_max_dim_panic() {
    let complex = Arc::new(create_line_complex()); // Max dim is 1 (edge)
    let weights = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).unwrap(); // 1-chain on (0,1)
    let chain = Chain::new(complex.clone(), 1, weights);

    complex.coboundary(&chain);
}

#[test]
fn test_boundary_inner_loop_breaks_after_match_with_multicolumn_chain() {
    // Triangle: 3 vertices, 3 edges, 1 face. A 1-chain over multiple edges
    // forces the inner column-search loop to find a match and `break` early
    // (before exhausting all chain columns) for several boundary rows.
    let complex = Arc::new(deep_causality_topology::utils_tests::create_triangle_complex());

    // 1-chain: 1*(0,1) + 1*(0,2) + 1*(1,2) — weights over all three edges.
    let weights = CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0), (0, 1, 1.0), (0, 2, 1.0)]).unwrap();
    let chain = Chain::new(complex.clone(), 1, weights);

    let boundary_chain = complex.boundary(&chain);
    assert_eq!(boundary_chain.grade(), 0);

    // ∂(e01 + e02 + e12) with the d1 orientation in `create_triangle_complex`:
    //   e01 = v1 - v0, e02 = v2 - v0, e12 = v2 - v1.
    // Summing: v0 = -1 - 1 = -2, v1 = +1 - 1 = 0, v2 = +1 + 1 = +2.
    // The chain is NOT a cycle (the cycle is e01 + e12 - e02), so the boundary
    // is -2·v0 + 2·v2 — two surviving nonzero rows, the v1 row cancels out.
    let expected = CsrMatrix::from_triplets(1, 3, &[(0, 0, -2.0), (0, 2, 2.0)]).unwrap();
    assert_eq!(boundary_chain.weights(), &expected);
}
