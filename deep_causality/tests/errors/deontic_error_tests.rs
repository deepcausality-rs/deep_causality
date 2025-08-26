/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::DeonticError;
use std::error::Error;
use ultragraph::GraphError;

#[test]
fn test_failed_to_add_teloid() {
    let error = DeonticError::FailedToAddTeloid;
    assert_eq!(
        format!("{}", error),
        "Failed to add a new teloid to the graph."
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
    assert!(error.source().is_none());
}

#[test]
fn test_failed_to_add_edge() {
    let error = DeonticError::FailedToAddEdge(1, 2);
    assert_eq!(
        format!("{}", error),
        "Edge from 1 to 2 could not be created; a node may not exist or the edge already exists."
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
    assert!(error.source().is_none());
}

#[test]
fn test_graph_not_frozen() {
    let error = DeonticError::GraphNotFrozen;
    assert_eq!(
        format!("{}", error),
        "Deontic inference failed: The TeloidGraph must be frozen before evaluation."
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
    assert!(error.source().is_none());
}

#[test]
fn test_graph_is_frozen() {
    let error = DeonticError::GraphIsFrozen;
    assert_eq!(
        format!("{}", error),
        "Deontic inference failed: The TeloidGraph is frozen and cannot be modified."
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
    assert!(error.source().is_none());
}

#[test]
fn test_graph_is_cyclic() {
    let error = DeonticError::GraphIsCyclic;
    assert_eq!(
        format!("{}", error),
        "Deontic inference failed: The TeloidGraph contains a cycle and is invalid."
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
    assert!(error.source().is_none());
}

#[test]
fn test_teloid_not_found() {
    let error = DeonticError::TeloidNotFound { id: 42 };
    assert_eq!(
        format!("{}", error),
        "Deontic inference failed: Teloid with ID 42 not found in store."
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
    assert!(error.source().is_none());
}

#[test]
fn test_inconclusive_verdict() {
    let error = DeonticError::InconclusiveVerdict;
    assert_eq!(
        format!("{}", error),
        "Deontic inference failed: The final set of active norms was inconclusive."
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
    assert!(error.source().is_none());
}

#[test]
fn test_no_relevant_norms_found() {
    let error = DeonticError::NoRelevantNormsFound;
    assert_eq!(
        format!("{}", error),
        "No relevant norms found, so the action cannot be decided. Please check if you have added the correct tags."
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
    assert!(error.source().is_none());
}

#[test]
fn test_missing_context() {
    let error = DeonticError::MissingContext;
    assert_eq!(
        format!("{}", error),
        "Deontic inference failed: The CausalState is missing a context."
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
    assert!(error.source().is_none());
}

#[test]
fn test_graph_error() {
    let inner_error = GraphError::NodeNotFound(10);
    let error = DeonticError::GraphError(inner_error);
    assert_eq!(
        format!("{}", error),
        format!(
            "A graph operation failed during deontic inference: {}",
            inner_error
        )
    );
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);

    assert!(error.source().is_some());
    assert_eq!(error.source().unwrap().to_string(), inner_error.to_string());
}

#[test]
fn test_from_graph_error() {
    // Test GraphIsFrozen
    let graph_error_frozen = GraphError::GraphIsFrozen;
    let deontic_error: DeonticError = graph_error_frozen.into();
    assert_eq!(deontic_error, DeonticError::GraphIsFrozen);

    // Test GraphNotFrozen
    let graph_error_not_frozen = GraphError::GraphNotFrozen;
    let deontic_error: DeonticError = graph_error_not_frozen.into();
    assert_eq!(deontic_error, DeonticError::GraphNotFrozen);

    // Test GraphContainsCycle
    let graph_error_cyclic = GraphError::GraphContainsCycle;
    let deontic_error: DeonticError = graph_error_cyclic.into();
    assert_eq!(deontic_error, DeonticError::GraphIsCyclic);

    // Test EdgeCreationError
    let graph_error_edge = GraphError::EdgeCreationError {
        source: 1,
        target: 2,
    };
    let deontic_error: DeonticError = graph_error_edge.into();
    assert_eq!(deontic_error, DeonticError::FailedToAddEdge(1, 2));

    // Test other GraphError variants are wrapped
    let other_graph_error = GraphError::NodeNotFound(10);
    let deontic_error: DeonticError = other_graph_error.into();
    assert_eq!(deontic_error, DeonticError::GraphError(other_graph_error));

    let another_graph_error = GraphError::NodeNotFound(20);
    let deontic_error: DeonticError = another_graph_error.into();
    assert_eq!(deontic_error, DeonticError::GraphError(another_graph_error));
}
