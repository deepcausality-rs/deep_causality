/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphWitness};

#[test]
fn test_graph_functor() {
    let data = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let graph = Graph::new(3, data, 0).unwrap();

    let mapped_graph = GraphWitness::fmap(graph, |x| x * 2);

    assert_eq!(mapped_graph.data().as_slice(), &[2, 4, 6]);
}

#[test]
fn test_graph_extract() {
    let data = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let graph = Graph::new(3, data, 2).unwrap(); // Cursor at 2

    let val = GraphWitness::extract(&graph);
    assert_eq!(val, 30);
}

#[test]
fn test_graph_extend() {
    let data = CausalTensor::new(vec![1, 1, 1], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 0).unwrap();
    // Add edges: 0-1, 1-2
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(1, 2).unwrap();

    // Extend: sum of neighbor values (using extract on neighbors would be complex here,
    // so let's just use the graph structure available in the view)
    let extended_graph = GraphWitness::extend(&graph, |w: &Graph<i32>| {
        let current_idx = w.cursor();
        let neighbors = w.neighbors(current_idx).unwrap();
        let mut sum = 0;
        for &n in neighbors {
            // In a real scenario, we might navigate to neighbor and extract,
            // but here we just access data directly for simplicity of test
            sum += w.data().as_slice()[n];
        }
        sum
    });

    // Node 0: neighbors [1] -> val 1
    // Node 1: neighbors [0, 2] -> val 1 + 1 = 2
    // Node 2: neighbors [1] -> val 1
    assert_eq!(extended_graph.data().as_slice(), &[1, 2, 1]);
}

#[test]
fn comonad_right_identity_holds_for_nonzero_focus() {
    // extract(extend(w, f)) == f(w) must hold when the focus is not node 0.
    let data = CausalTensor::new(vec![3, 4, 5], vec![3]).unwrap();
    let graph = Graph::new(3, data, 2).unwrap();
    let f = |w: &Graph<i32>| w.data().as_slice()[w.cursor()] + 100;
    let extended = GraphWitness::extend(&graph, f);
    assert_eq!(extended.cursor(), 2);
    assert_eq!(GraphWitness::extract(&extended), 105);
}

#[test]
fn comonad_associativity_law() {
    // extend(extend(w, g), f) == extend(w, |w'| f(&extend(w', g)))
    let data = CausalTensor::new(vec![5, 7, 11], vec![3]).unwrap();
    let w = Graph::new(3, data, 1).unwrap();
    let g = |w: &Graph<i32>| w.data().as_slice()[w.cursor()] + 1;
    let f = |w: &Graph<i32>| w.data().as_slice()[w.cursor()] * 10;

    let lhs = GraphWitness::extend(&GraphWitness::extend(&w, g), f);
    let rhs = GraphWitness::extend(&w, |wp: &Graph<i32>| f(&GraphWitness::extend(wp, g)));

    assert_eq!(lhs.data().as_slice(), rhs.data().as_slice());
}
