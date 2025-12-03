/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::ExecutableEdge;

#[test]
fn test_executable_edge_new() {
    let edge = ExecutableEdge::new(1, 2);
    assert_eq!(edge.from(), 1);
    assert_eq!(edge.to(), 2);
}

#[test]
fn test_executable_edge_getters() {
    let edge = ExecutableEdge::new(10, 20);
    assert_eq!(edge.from(), 10);
    assert_eq!(edge.to(), 20);
}

#[test]
fn test_executable_edge_display() {
    let edge = ExecutableEdge::new(1, 2);
    assert_eq!(format!("{}", edge), "ExecutableEdge(from: 1, to: 2)");
}

#[test]
fn test_executable_edge_clone_debug() {
    let edge = ExecutableEdge::new(1, 2);
    let edge_clone = edge.clone();
    assert_eq!(edge.from(), edge_clone.from());
    assert_eq!(edge.to(), edge_clone.to());
    assert_eq!(format!("{:?}", edge), "ExecutableEdge { from: 1, to: 2 }");
}
