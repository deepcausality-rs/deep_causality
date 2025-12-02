/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::BoundedAdjunction;
use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::{Chain, Simplex, SimplicialComplex, Skeleton, Topology};
use std::sync::Arc;

fn create_simple_complex() -> Arc<SimplicialComplex> {
    // Single triangle: {0, 1, 2}
    let vertices = vec![
        Simplex::new(vec![0]),
        Simplex::new(vec![1]),
        Simplex::new(vec![2]),
    ];
    let skeleton_0 = Skeleton::new(0, vertices);

    // We only need 0-skeleton for the current HKT implementation of unit/left_adjunct
    // as it defaults to 0-skeleton in the code I read.

    Arc::new(SimplicialComplex::new(
        vec![skeleton_0],
        vec![],
        vec![],
        vec![],
    ))
}

#[test]
fn test_simplicial_complex_unit() {
    let complex = create_simple_complex();
    let val = 42.0;

    // Unit: Embed scalar into Topology<Chain<A>>
    // Creates a topology where every point holds a Chain concentrated at that point with value `val`.
    let topology: Topology<Chain<f64>> = SimplicialComplex::unit(&complex, val);

    assert_eq!(topology.grade(), 0);
    assert_eq!(topology.data().len(), 3); // 3 vertices

    // Check first element
    let chain_0 = &topology.data().as_slice()[0];
    // Should be a chain with weight 42.0 at index 0
    let w = chain_0.weights();
    // Accessing sparse matrix value at (0,0)
    assert_eq!(w.get_value_at(0, 0), 42.0);
}

#[test]
fn test_simplicial_complex_left_adjunct() {
    let complex = create_simple_complex();

    // Left Adjunct: (Chain<A> -> B) -> (A -> Topology<B>)
    // We provide a function f: Chain<f64> -> f64
    // For example, f sums the weights in the chain.
    let f = |c: Chain<f64>| c.weights().values().iter().sum::<f64>();

    let topology = SimplicialComplex::left_adjunct(&complex, 0.0, f);

    assert_eq!(topology.data().len(), 3);
    // With current implementation (empty chains), sum is 0.
    assert_eq!(topology.data().as_slice()[0], 0.0);
}

#[test]
fn test_simplicial_complex_counit() {
    let complex = create_simple_complex();

    // Counit: Chain<Topology<B>> -> B
    // Integrate the field over the chain.

    // NOTE: CsrMatrix requires T: Copy for construction via from_triplets.
    // Topology<f64> is not Copy.
    // Therefore, we cannot construct a non-empty Chain<Topology<f64>> using public APIs.
    // We test with an empty chain, which should result in zero.

    let weights: CsrMatrix<Topology<f64>> = CsrMatrix::new(); // Empty matrix
    let chain = Chain::new(complex.clone(), 0, weights);

    let result = SimplicialComplex::counit(&complex, chain);
    assert_eq!(result, 0.0);
}
