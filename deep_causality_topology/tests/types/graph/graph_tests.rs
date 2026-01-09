/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, TopologyError, TopologyErrorEnum};

/// Helper to create a simple graph
fn create_simple_graph() -> Graph<f64> {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    Graph::new(3, data, 0).unwrap()
}

#[test]
fn test_graph_new_success() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let graph = Graph::new(3, data, 0);
    assert!(graph.is_ok());
    let g = graph.unwrap();
    assert_eq!(g.num_vertices(), 3);
    assert_eq!(g.num_edges(), 0);
}

#[test]
fn test_graph_new_zero_vertices() {
    let data = CausalTensor::new(vec![], vec![0]).unwrap();
    let result = Graph::<f64>::new(0, data, 0);
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::InvalidInput(msg))) => {
            assert!(msg.contains("at least one vertex"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_graph_new_data_size_mismatch() {
    let data = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let result = Graph::new(3, data, 0); // 3 vertices but only 2 data points
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::InvalidInput(msg))) => {
            assert!(msg.contains("Data size must match"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_graph_new_cursor_out_of_bounds() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let result = Graph::new(3, data, 5); // cursor 5 > num_vertices 3
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::IndexOutOfBounds(msg))) => {
            assert!(msg.contains("cursor out of bounds"));
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}

#[test]
fn test_graph_add_edge_success() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 0).unwrap();

    let result = graph.add_edge(0, 1);
    assert!(result.is_ok());
    assert!(result.unwrap()); // Edge was added
    assert_eq!(graph.num_edges(), 1);
}

#[test]
fn test_graph_add_edge_duplicate() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 0).unwrap();

    graph.add_edge(0, 1).unwrap();
    let result = graph.add_edge(0, 1);
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Edge already exists
    assert_eq!(graph.num_edges(), 1); // Still only 1 edge
}

#[test]
fn test_graph_add_edge_self_loop() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 0).unwrap();

    let result = graph.add_edge(0, 0);
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::GraphError(msg))) => {
            assert!(msg.contains("Self-loops are not allowed"));
        }
        _ => panic!("Expected GraphError"),
    }
}

#[test]
fn test_graph_add_edge_out_of_bounds() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 0).unwrap();

    let result = graph.add_edge(0, 5);
    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::GraphError(msg))) => {
            assert!(msg.contains("out of bounds"));
        }
        _ => panic!("Expected GraphError"),
    }
}

#[test]
fn test_graph_has_edge() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 0).unwrap();

    graph.add_edge(0, 1).unwrap();
    assert!(graph.has_edge(0, 1).unwrap());
    assert!(graph.has_edge(1, 0).unwrap()); // Undirected
    assert!(!graph.has_edge(0, 2).unwrap());
}

#[test]
fn test_graph_neighbors() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 0).unwrap();

    graph.add_edge(0, 1).unwrap();
    graph.add_edge(0, 2).unwrap();

    let neighbors = graph.neighbors(0).unwrap();
    assert_eq!(neighbors.len(), 2);
    assert!(neighbors.contains(&1));
    assert!(neighbors.contains(&2));
}

// =============================================================================
// add_edge error paths
// =============================================================================

#[test]
fn test_add_edge_success() {
    let mut graph = create_simple_graph();
    let result = graph.add_edge(0, 1);
    assert!(result.is_ok());
    assert!(result.unwrap(), "New edge should return true");
}

#[test]
fn test_add_edge_duplicate() {
    let mut graph = create_simple_graph();
    graph.add_edge(0, 1).unwrap();

    // Adding same edge again should return false (not an error)
    let result = graph.add_edge(0, 1);
    assert!(result.is_ok());
    assert!(!result.unwrap(), "Duplicate edge should return false");
}

#[test]
fn test_add_edge_out_of_bounds() {
    let mut graph = create_simple_graph();
    let result = graph.add_edge(0, 10); // v=10 is out of bounds

    assert!(result.is_err());
    match result.unwrap_err().0 {
        TopologyErrorEnum::GraphError(msg) => {
            assert!(
                msg.contains("out of bounds"),
                "Should mention out of bounds: {}",
                msg
            );
        }
        e => panic!("Expected GraphError, got {:?}", e),
    }
}

#[test]
fn test_add_edge_self_loop() {
    let mut graph = create_simple_graph();
    let result = graph.add_edge(1, 1); // Self-loop

    assert!(result.is_err());
    match result.unwrap_err().0 {
        TopologyErrorEnum::GraphError(msg) => {
            assert!(
                msg.contains("Self-loops"),
                "Should mention self-loops: {}",
                msg
            );
        }
        e => panic!("Expected GraphError, got {:?}", e),
    }
}

// =============================================================================
// has_edge error paths
// =============================================================================

#[test]
fn test_has_edge_success() {
    let mut graph = create_simple_graph();
    graph.add_edge(0, 2).unwrap();

    assert!(graph.has_edge(0, 2).unwrap());
    assert!(!graph.has_edge(0, 1).unwrap());
}

#[test]
fn test_has_edge_out_of_bounds() {
    let graph = create_simple_graph();
    let result = graph.has_edge(100, 0);

    assert!(result.is_err());
    match result.unwrap_err().0 {
        TopologyErrorEnum::GraphError(msg) => {
            assert!(msg.contains("out of bounds"));
        }
        e => panic!("Expected GraphError, got {:?}", e),
    }
}

// =============================================================================
// neighbors error paths
// =============================================================================

#[test]
fn test_neighbors_success() {
    let mut graph = create_simple_graph();
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(0, 2).unwrap();

    let neighbors = graph.neighbors(0).unwrap();
    assert_eq!(neighbors.len(), 2);
}

#[test]
fn test_neighbors_out_of_bounds() {
    let graph = create_simple_graph();
    let result = graph.neighbors(50);

    assert!(result.is_err());
    match result.unwrap_err().0 {
        TopologyErrorEnum::GraphError(msg) => {
            assert!(msg.contains("out of bounds"));
        }
        e => panic!("Expected GraphError, got {:?}", e),
    }
}

// =============================================================================
// Display and other coverage
// =============================================================================

#[test]
fn test_graph_display() {
    let graph = create_simple_graph();
    let display_str = format!("{}", graph);

    assert!(display_str.contains("Graph"));
    assert!(display_str.contains("3")); // num vertices
}
