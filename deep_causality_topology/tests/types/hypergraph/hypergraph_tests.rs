/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Hypergraph, HypergraphTopology, TopologyError, TopologyErrorEnum};

/// Helper to create a simple hypergraph
fn create_simple_hypergraph() -> Hypergraph<f64> {
    // 3 nodes, 2 hyperedges
    // Hyperedge 0: nodes {0, 1}
    // Hyperedge 1: nodes {1, 2}
    let triplets = vec![
        (0, 0, 1i8), // node 0 in hyperedge 0
        (1, 0, 1i8), // node 1 in hyperedge 0
        (1, 1, 1i8), // node 1 in hyperedge 1
        (2, 1, 1i8), // node 2 in hyperedge 1
    ];
    let incidence = CsrMatrix::from_triplets(3, 2, &triplets).unwrap();
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();

    // API: new(incidence, data, cursor)
    Hypergraph::new(incidence, data, 0).unwrap()
}

#[test]
fn test_hypergraph_new_success() {
    // 3 nodes, 2 hyperedges
    // Hyperedge 0: nodes {0, 1}
    // Hyperedge 1: nodes {1, 2}
    let incidence = CsrMatrix::from_triplets(
        3,
        2,
        &[
            (0, 0, 1i8), // node 0 in hyperedge 0
            (1, 0, 1),   // node 1 in hyperedge 0
            (1, 1, 1),   // node 1 in hyperedge 1
            (2, 1, 1),   // node 2 in hyperedge 1
        ],
    )
    .unwrap();

    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let result = Hypergraph::new(incidence, data, 0);
    assert!(result.is_ok());

    let hg = result.unwrap();
    assert_eq!(hg.num_nodes(), 3);
    assert_eq!(hg.num_hyperedges(), 2);
}

#[test]
fn test_hypergraph_new_data_size_mismatch() {
    let incidence = CsrMatrix::from_triplets(3, 2, &[(0, 0, 1i8), (1, 0, 1)]).unwrap();
    let data = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap(); // Only 2 but need 3

    let result = Hypergraph::new(incidence, data, 0);
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::InvalidInput(msg))) => {
            assert!(msg.contains("Data size must match"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_hypergraph_new_cursor_out_of_bounds() {
    let incidence = CsrMatrix::from_triplets(3, 2, &[(0, 0, 1i8), (1, 0, 1)]).unwrap();
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();

    let result = Hypergraph::new(incidence, data, 5); // cursor 5 > num_nodes 3
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::IndexOutOfBounds(msg))) => {
            assert!(msg.contains("cursor out of bounds"));
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_hypergraph_invalid_incidence_values() {
    // Incidence matrix should only have 0 or 1
    let incidence = CsrMatrix::from_triplets(3, 2, &[(0, 0, 2i8), (1, 0, 1)]).unwrap();
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();

    let result = Hypergraph::new(incidence, data, 0);
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::HypergraphError(msg))) => {
            assert!(msg.contains("must be 0 or 1"));
        }
        _ => panic!("Expected HypergraphError"),
    }
}

#[test]
fn test_hypergraph_nodes_in_hyperedge() {
    let incidence = CsrMatrix::from_triplets(
        4,
        2,
        &[
            (0, 0, 1i8),
            (1, 0, 1),
            (2, 0, 1), // Hyperedge 0 has nodes 0, 1, 2
            (2, 1, 1),
            (3, 1, 1), // Hyperedge 1 has nodes 2, 3
        ],
    )
    .unwrap();

    let data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let hg = Hypergraph::new(incidence, data, 0).unwrap();

    let nodes0 = hg.nodes_in_hyperedge(0).unwrap();
    assert_eq!(nodes0.len(), 3);
    assert!(nodes0.contains(&0));
    assert!(nodes0.contains(&1));
    assert!(nodes0.contains(&2));

    let nodes1 = hg.nodes_in_hyperedge(1).unwrap();
    assert_eq!(nodes1.len(), 2);
    assert!(nodes1.contains(&2));
    assert!(nodes1.contains(&3));
}

#[test]
fn test_hypergraph_hyperedges_on_node() {
    let incidence = CsrMatrix::from_triplets(
        4,
        3,
        &[
            (0, 0, 1i8),
            (1, 0, 1), // Hyperedge 0: nodes 0, 1
            (1, 1, 1),
            (2, 1, 1), // Hyperedge 1: nodes 1, 2
            (2, 2, 1),
            (3, 2, 1), // Hyperedge 2: nodes 2, 3
        ],
    )
    .unwrap();

    let data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let hg = Hypergraph::new(incidence, data, 0).unwrap();

    let edges1 = hg.hyperedges_on_node(1).unwrap();
    assert_eq!(edges1.len(), 2);
    assert!(edges1.contains(&0));
    assert!(edges1.contains(&1));

    let edges2 = hg.hyperedges_on_node(2).unwrap();
    assert_eq!(edges2.len(), 2);
    assert!(edges2.contains(&1));
    assert!(edges2.contains(&2));
}

// =============================================================================
// nodes_in_hyperedge error paths
// =============================================================================

#[test]
fn test_nodes_in_hyperedge_success() {
    let hg = create_simple_hypergraph();

    let nodes0 = hg.nodes_in_hyperedge(0).unwrap();
    assert!(nodes0.contains(&0));
    assert!(nodes0.contains(&1));

    let nodes1 = hg.nodes_in_hyperedge(1).unwrap();
    assert!(nodes1.contains(&1));
    assert!(nodes1.contains(&2));
}

#[test]
fn test_nodes_in_hyperedge_out_of_bounds() {
    let hg = create_simple_hypergraph();
    let result = hg.nodes_in_hyperedge(10); // Out of bounds

    assert!(result.is_err());
    match result.unwrap_err().0 {
        TopologyErrorEnum::HypergraphError(msg) => {
            assert!(
                msg.contains("out of bounds"),
                "Should mention out of bounds: {}",
                msg
            );
        }
        e => panic!("Expected HypergraphError, got {:?}", e),
    }
}

// =============================================================================
// hyperedges_on_node error paths
// =============================================================================

#[test]
fn test_hyperedges_on_node_success() {
    let hg = create_simple_hypergraph();

    // Node 1 is in both hyperedges
    let edges = hg.hyperedges_on_node(1).unwrap();
    assert_eq!(edges.len(), 2);

    // Node 0 is only in hyperedge 0
    let edges0 = hg.hyperedges_on_node(0).unwrap();
    assert_eq!(edges0.len(), 1);
    assert!(edges0.contains(&0));
}

#[test]
fn test_hyperedges_on_node_out_of_bounds() {
    let hg = create_simple_hypergraph();
    let result = hg.hyperedges_on_node(100); // Out of bounds

    assert!(result.is_err());
    match result.unwrap_err().0 {
        TopologyErrorEnum::HypergraphError(msg) => {
            assert!(msg.contains("out of bounds"));
        }
        e => panic!("Expected HypergraphError, got {:?}", e),
    }
}

// =============================================================================
// num_hyperedges coverage
// =============================================================================

#[test]
fn test_num_hyperedges() {
    let hg = create_simple_hypergraph();
    assert_eq!(hg.num_hyperedges(), 2);
}

// =============================================================================
// cursor coverage
// =============================================================================

#[test]
fn test_hypergraph_cursor() {
    let hg = create_simple_hypergraph();
    assert_eq!(hg.cursor(), 0);
}
