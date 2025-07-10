/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::error::Error;
use ultragraph::GraphError;

#[test]
fn test_node_not_found_error() {
    let error = GraphError::NodeNotFound(42);
    assert_eq!(
        format!("{error}"),
        "Node with index 42 not found; it may be out of bounds or have been removed."
    );
    assert!(error.source().is_none());
}

#[test]
fn test_edge_creation_error() {
    let error = GraphError::EdgeCreationError {
        source: 1,
        target: 2,
    };
    assert_eq!(
        format!("{error}"),
        "Edge from 1 to 2 could not be created; a node may not exist or the edge already exists."
    );
    assert!(error.source().is_none());
}

#[test]
fn test_edge_not_found_error() {
    let error = GraphError::EdgeNotFoundError {
        source: 10,
        target: 20,
    };
    assert_eq!(format!("{error}"), "Edge from 10 to 20 not found.");
    assert!(error.source().is_none());
}

#[test]
fn test_graph_contains_cycle_error() {
    let error = GraphError::GraphContainsCycle;
    assert_eq!(
        format!("{error}"),
        "Operation failed because the graph contains a cycle."
    );
    assert!(error.source().is_none());
}

#[test]
fn test_graph_not_frozen_error() {
    let error = GraphError::GraphNotFrozen;
    assert_eq!(
        format!("{error}"),
        "Operation not possible because the graph is not frozen. Call graph.freeze() first."
    );
    assert!(error.source().is_none());
}

#[test]
fn test_graph_is_frozen_error() {
    let error = GraphError::GraphIsFrozen;
    assert_eq!(
        format!("{error}"),
        "Operation not possible because the graph is frozen and cannot be mutated. Call graph.unfreeze() first."
    );
    assert!(error.source().is_none());
}

#[test]
fn test_root_node_error() {
    let error = GraphError::RootNodeAlreadyExists;
    assert_eq!(format!("{error}"), "Root node already exists");
    assert!(error.source().is_none());
}

#[test]
fn test_error_traits() {
    let error1 = GraphError::NodeNotFound(5);
    let error2 = GraphError::NodeNotFound(5);
    let error3 = GraphError::NodeNotFound(6);

    // Test PartialEq
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);

    // Test Clone and Copy
    let cloned_error = error1;
    assert_eq!(error1, cloned_error);
    let copied_error = error1;
    assert_eq!(error1, copied_error);
}

/// Tests the creation and content of the AlgorithmError variant.
#[test]
fn test_algorithm_error_creation() {
    let err_msg = "Pathfinding failed: graph is disconnected";
    let error = GraphError::AlgorithmError(err_msg);

    // You can use a `match` or `if let` to inspect the enum variant and its data.
    if let GraphError::AlgorithmError(msg) = error {
        assert_eq!(
            msg, err_msg,
            "The error message should be stored correctly."
        );
    } else {
        panic!("Expected GraphError::AlgorithmError, but got a different variant.");
    }
}

/// Tests the `Display` trait implementation for user-friendly output.
#[test]
fn test_algorithm_error_display_formatting() {
    let err_msg = "Dijkstra's algorithm requires non-negative weights";
    let error = GraphError::AlgorithmError(err_msg);
    let expected_display_msg = format!("AlgorithmError: {err_msg}");

    assert_eq!(error.to_string(), expected_display_msg);
}

/// Tests the `Debug` trait implementation for developer-focused output.
#[test]
fn test_algorithm_error_debug_formatting() {
    let err_msg = "A cycle was detected in a directed acyclic graph (DAG)";
    let error = GraphError::AlgorithmError(err_msg);
    // The `Debug` format is derived automatically and includes the variant name.
    let expected_debug_msg = format!("AlgorithmError(\"{err_msg}\")");

    assert_eq!(format!("{error:?}"), expected_debug_msg);
}

/// Demonstrates how to check for a specific error variant within a `Result`.
#[test]
fn test_algorithm_error_in_a_result() {
    let err_msg = "Invalid start node for traversal";

    // A sample function that returns our specific error.
    fn find_path() -> Result<Vec<usize>, GraphError> {
        Err(GraphError::AlgorithmError(
            "Invalid start node for traversal",
        ))
    }

    let result = find_path();
    assert!(result.is_err());

    // Match on the error to confirm its type and contents.
    match result {
        Err(GraphError::AlgorithmError(msg)) => {
            assert_eq!(msg, err_msg);
        }
        _ => panic!("Expected an AlgorithmError variant."),
    }
}
