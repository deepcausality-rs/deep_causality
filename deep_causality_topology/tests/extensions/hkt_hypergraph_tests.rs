/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor};
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
    let extended = HypergraphWitness::extend(&hypergraph, |w: &Hypergraph<i32>| {
        let current_node = w.cursor();
        w.hyperedges_on_node(current_node).unwrap().len() as i32
    });

    // All nodes belong to 1 hyperedge
    assert_eq!(extended.data().as_slice(), &[1, 1, 1]);
}

#[test]
fn comonad_right_identity_holds_for_nonzero_focus() {
    // extract(extend(w, f)) == f(w) must hold when the focus is not node 0.
    let incidence = CsrMatrix::from_triplets(3, 1, &[(0, 0, 1), (1, 0, 1), (2, 0, 1)]).unwrap();
    let data = CausalTensor::new(vec![3, 4, 5], vec![3]).unwrap();
    let h = Hypergraph::new(incidence, data, 2).unwrap();
    let f = |w: &Hypergraph<i32>| w.data().as_slice()[w.cursor()] + 100;
    let extended = HypergraphWitness::extend(&h, f);
    assert_eq!(extended.cursor(), 2);
    assert_eq!(HypergraphWitness::extract(&extended), 105);
}

#[test]
fn comonad_associativity_law() {
    // extend(extend(w, g), f) == extend(w, |w'| f(&extend(w', g)))
    let incidence = CsrMatrix::from_triplets(3, 1, &[(0, 0, 1), (1, 0, 1), (2, 0, 1)]).unwrap();
    let data = CausalTensor::new(vec![5, 7, 11], vec![3]).unwrap();
    let w = Hypergraph::new(incidence, data, 1).unwrap();
    let g = |w: &Hypergraph<i32>| w.data().as_slice()[w.cursor()] + 1;
    let f = |w: &Hypergraph<i32>| w.data().as_slice()[w.cursor()] * 10;

    let lhs = HypergraphWitness::extend(&HypergraphWitness::extend(&w, g), f);
    let rhs = HypergraphWitness::extend(&w, |wp: &Hypergraph<i32>| {
        f(&HypergraphWitness::extend(wp, g))
    });

    assert_eq!(lhs.data().as_slice(), rhs.data().as_slice());
}
