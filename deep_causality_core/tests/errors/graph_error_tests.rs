/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::GraphError;

#[test]
fn test_graph_error_display() {
    let err1 = GraphError::StartNodeOutOfBounds(5);
    assert_eq!(format!("{}", err1), "Start node index 5 out of bounds");

    let err2 = GraphError::MaxStepsExceeded(100);
    assert_eq!(
        format!("{}", err2),
        "Execution exceeded max_steps limit of 100"
    );

    let err3 = GraphError::GraphExecutionProducedNoResult;
    assert_eq!(format!("{}", err3), "Graph execution produced no result");
}

#[test]
fn test_graph_error_traits() {
    let err = GraphError::StartNodeOutOfBounds(1);
    let err_clone = err.clone();
    assert_eq!(err, err_clone);
    assert_eq!(format!("{:?}", err), "StartNodeOutOfBounds(1)");
}
