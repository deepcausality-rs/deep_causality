/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use ultragraph::*;

#[test]
fn test_new() {
    let g = CausaloidGraph::new(0);
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_new_with_capacity() {
    let g = CausaloidGraph::new_with_capacity(0, 10);
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_default() {
    let g = CausaloidGraph::default();
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_get_graph() {
    let g = CausaloidGraph::new(0);

    let size = g.size();
    assert_eq!(size, 0);

    let graph = g.get_graph();
    assert_eq!(graph.number_edges(), 0);
    assert_eq!(graph.number_nodes(), 0);
}

#[test]
fn test_id() {
    let g = CausaloidGraph::new(0);
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert_eq!(g.id(), 0);
}
