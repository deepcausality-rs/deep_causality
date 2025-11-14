/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use ultragraph::*;

use deep_causality::utils_test::test_utils;

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
fn test_add_edge() {
    let mut registry = CausaloidRegistry::new();
    let causaloid_a = test_utils::get_test_causaloid_deterministic(1);
    let causaloid_b = test_utils::get_test_causaloid_deterministic(2);

    let id_a = registry.register(causaloid_a);
    let id_b = registry.register(causaloid_b);

    let mut g = CausaloidGraph::new(0);

    let idx_a = g.add_causaloid(id_a).expect("Failed to add causaloid");
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let idx_b = g.add_causaloid(id_b).expect("Failed to add causaloid");
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let res = g.add_edge(idx_a, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(contains_edge);
}

#[test]
fn test_add_edg_with_weight() {
    let mut registry = CausaloidRegistry::new();
    let causaloid_a = test_utils::get_test_causaloid_deterministic(1);
    let causaloid_b = test_utils::get_test_causaloid_deterministic(2);

    let id_a = registry.register(causaloid_a);
    let id_b = registry.register(causaloid_b);

    let mut g = CausaloidGraph::new(0);

    let idx_a = g.add_causaloid(id_a).expect("Failed to add causaloid");
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let idx_b = g.add_causaloid(id_b).expect("Failed to add causaloid");
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let weight = 1;
    let res = g.add_edg_with_weight(idx_a, idx_b, weight);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(contains_edge);
}

#[test]
fn test_remove_edge() {
    let mut registry = CausaloidRegistry::new();
    let causaloid_a = test_utils::get_test_causaloid_deterministic(1);
    let causaloid_b = test_utils::get_test_causaloid_deterministic(2);

    let id_a = registry.register(causaloid_a);
    let id_b = registry.register(causaloid_b);

    let mut g = CausaloidGraph::new(0);

    let idx_a = g.add_causaloid(id_a).expect("Failed to add causaloid");
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let idx_b = g.add_causaloid(id_b).expect("Failed to add causaloid");
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let res = g.add_edge(idx_a, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(contains_edge);

    let res = g.remove_edge(idx_a, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(!contains_edge);
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
