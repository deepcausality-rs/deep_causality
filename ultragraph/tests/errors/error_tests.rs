use std::error::Error;
use ultragraph::GraphError;

#[test]
fn test_node_not_found_error() {
    let error = GraphError::NodeNotFound(42);
    assert_eq!(
        format!("{}", error),
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
        format!("{}", error),
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
    assert_eq!(format!("{}", error), "Edge from 10 to 20 not found.");
    assert!(error.source().is_none());
}

#[test]
fn test_graph_contains_cycle_error() {
    let error = GraphError::GraphContainsCycle;
    assert_eq!(
        format!("{}", error),
        "Operation failed because the graph contains a cycle."
    );
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
    let cloned_error = error1.clone();
    assert_eq!(error1, cloned_error);
    let copied_error = error1;
    assert_eq!(error1, copied_error);
}
