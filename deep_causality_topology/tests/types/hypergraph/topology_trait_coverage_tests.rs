/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GraphTopology, Hypergraph, HypergraphTopology};

fn create_simple_hypergraph() -> Hypergraph<f64> {
    // Hyperedge 0: {0, 1}
    // Hyperedge 1: {1, 2}
    let incidence =
        CsrMatrix::from_triplets(3, 2, &[(0, 0, 1i8), (1, 0, 1), (1, 1, 1), (2, 1, 1)]).unwrap();
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    Hypergraph::new(incidence, data, 0).unwrap()
}

// `Hypergraph::num_nodes` and `Hypergraph::num_hyperedges` inherent getters
// shadow the corresponding `GraphTopology` / `HypergraphTopology` trait
// methods. These tests reach the trait bodies via fully-qualified syntax.
#[test]
fn test_graph_topology_num_nodes_trait_qualified() {
    let hg = create_simple_hypergraph();
    assert_eq!(<Hypergraph<f64> as GraphTopology>::num_nodes(&hg), 3);
    assert_eq!(<Hypergraph<f64> as GraphTopology>::num_edges(&hg), 2);

    // Exercise get_neighbors so the neighbor-insertion path runs.
    let neighbors = <Hypergraph<f64> as GraphTopology>::get_neighbors(&hg, 1).unwrap();
    assert!(neighbors.contains(&0));
    assert!(neighbors.contains(&2));
}

#[test]
fn test_hypergraph_topology_num_hyperedges_trait_qualified() {
    let hg = create_simple_hypergraph();
    assert_eq!(
        <Hypergraph<f64> as HypergraphTopology>::num_hyperedges(&hg),
        2
    );
}
