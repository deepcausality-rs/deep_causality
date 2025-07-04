/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

#[test]
fn test_new() {
    let g: UltraGraph<i32> = UltraGraph::new();
    assert!(g.is_empty());
    assert!(!g.is_frozen());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_with_capacity() {
    let g: UltraGraph<i32> = UltraGraph::with_capacity(10, Some(20));
    assert!(g.is_empty());
    assert!(!g.is_frozen());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    // Note: Capacity itself is not exposed in the public API,
    // so we just check that the graph is created correctly.
}

#[test]
fn test_with_capacity_nodes_only() {
    let g: UltraGraph<i32> = UltraGraph::with_capacity(10, None);
    assert!(g.is_empty());
    assert!(!g.is_frozen());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}
