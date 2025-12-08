/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Hypergraph, HypergraphTopology};

fn create_simple_hypergraph() -> Hypergraph<f64> {
    // Hyperedge 0: {0, 1}
    // Hyperedge 1: {1, 2}
    let incidence =
        CsrMatrix::from_triplets(3, 2, &[(0, 0, 1i8), (1, 0, 1), (1, 1, 1), (2, 1, 1)]).unwrap();

    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    Hypergraph::new(incidence, data, 0).unwrap()
}

#[test]
fn test_hypergraph_hypergraph_topology() {
    let hg = create_simple_hypergraph();

    // num_hyperedges
    assert_eq!(hg.num_hyperedges(), 2);

    // nodes_in_hyperedge
    let nodes_h0 = hg.nodes_in_hyperedge(0).unwrap();
    assert_eq!(nodes_h0.len(), 2);
    assert!(nodes_h0.contains(&0));
    assert!(nodes_h0.contains(&1));

    let nodes_h1 = hg.nodes_in_hyperedge(1).unwrap();
    assert_eq!(nodes_h1.len(), 2);
    assert!(nodes_h1.contains(&1));
    assert!(nodes_h1.contains(&2));

    // Error cases
    assert!(hg.nodes_in_hyperedge(99).is_err());

    // hyperedges_on_node
    let edges_n0 = hg.hyperedges_on_node(0).unwrap();
    assert_eq!(edges_n0.len(), 1);
    assert_eq!(edges_n0[0], 0);

    let edges_n1 = hg.hyperedges_on_node(1).unwrap();
    assert_eq!(edges_n1.len(), 2);
    assert!(edges_n1.contains(&0));
    assert!(edges_n1.contains(&1));

    // Error cases
    assert!(hg.hyperedges_on_node(99).is_err());
}
