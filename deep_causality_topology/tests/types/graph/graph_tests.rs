/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, Graph, GraphTopology, TopologyError};

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
        Err(TopologyError::InvalidInput(msg)) => {
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
        Err(TopologyError::InvalidInput(msg)) => {
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
        Err(TopologyError::IndexOutOfBounds(msg)) => {
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
        Err(TopologyError::GraphError(msg)) => {
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
        Err(TopologyError::GraphError(msg)) => {
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

#[test]
fn test_graph_base_topology() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let mut graph = Graph::new(3, data, 0).unwrap();
    graph.add_edge(0, 1).unwrap();

    assert_eq!(graph.dimension(), 1);
    assert_eq!(graph.len(), 3);
    assert!(!graph.is_empty());
    assert_eq!(graph.num_elements_at_grade(0), Some(3)); // vertices
    assert_eq!(graph.num_elements_at_grade(1), Some(1)); // edges
    assert_eq!(graph.num_elements_at_grade(2), None);
}

#[test]
fn test_graph_topology() {
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let mut graph = Graph::new(4, data, 0).unwrap();
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(1, 2).unwrap();

    assert_eq!(graph.num_nodes(), 4);
    assert_eq!(graph.num_edges(), 2);
    assert!(graph.has_node(0));
    assert!(graph.has_node(3));
    assert!(!graph.has_node(5));

    let neighbors = graph.get_neighbors(1).unwrap();
    assert_eq!(neighbors.len(), 2);
    assert!(neighbors.contains(&0));
    assert!(neighbors.contains(&2));
}
