/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{BoundedComonad, Functor};
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Hypergraph, HypergraphTopology, HypergraphWitness};

#[test]
fn test_hypergraph_functor() {
    // 3 nodes, 2 hyperedges
    // H1: {0, 1}, H2: {1, 2}
    let incidence =
        CsrMatrix::from_triplets(3, 2, &[(0, 0, 1), (1, 0, 1), (1, 1, 1), (2, 1, 1)]).unwrap();
    let data = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let hypergraph = Hypergraph::new(incidence, data, 0).unwrap();

    let mapped = HypergraphWitness::fmap(hypergraph, |x| x * 10);

    assert_eq!(mapped.data().as_slice(), &[10, 20, 30]);
}

#[test]
fn test_hypergraph_extract() {
    let incidence = CsrMatrix::from_triplets(2, 1, &[(0, 0, 1), (1, 0, 1)]).unwrap();
    let data = CausalTensor::new(vec![100, 200], vec![2]).unwrap();
    let hypergraph = Hypergraph::new(incidence, data, 1).unwrap(); // Cursor at 1

    let val = HypergraphWitness::extract(&hypergraph);
    assert_eq!(val, 200);
}

#[test]
fn test_hypergraph_extend() {
    // 3 nodes, 1 hyperedge {0, 1, 2}
    let incidence = CsrMatrix::from_triplets(3, 1, &[(0, 0, 1), (1, 0, 1), (2, 0, 1)]).unwrap();
    let data = CausalTensor::new(vec![1, 1, 1], vec![3]).unwrap();
    let hypergraph = Hypergraph::new(incidence, data, 0).unwrap();

    // Extend: Count how many hyperedges the current node belongs to
    // (This is a structural property, not dependent on data values, but shows access to structure)
    let extended = HypergraphWitness::extend(&hypergraph, |w| {
        let current_node = w.cursor();
        w.hyperedges_on_node(current_node).unwrap().len() as i32
    });

    // All nodes belong to 1 hyperedge
    assert_eq!(extended.data().as_slice(), &[1, 1, 1]);
}
