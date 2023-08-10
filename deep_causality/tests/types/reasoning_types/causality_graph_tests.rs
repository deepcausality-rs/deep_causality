// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::*;
use deep_causality::utils::test_utils;

#[test]
fn test_new() {
    let g: CausaloidGraph<Causaloid<Dataoid, Spaceoid, Tempoid, SpaceTempoid>> = CausaloidGraph::new();
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_new_with_capacity() {
    let g: CausaloidGraph<Causaloid<Dataoid, Spaceoid, Tempoid, SpaceTempoid>> = CausaloidGraph::new_with_capacity(10);
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_default() {
    let g: CausaloidGraph<Causaloid<Dataoid, Spaceoid, Tempoid, SpaceTempoid>> = CausaloidGraph::default();
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_add_root_causaloid() {
    let mut g = CausaloidGraph::new();
    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);
}

#[test]
fn test_get_root_causaloid() {
    let mut g = CausaloidGraph::new();
    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    let causaloid = g.get_root_causaloid().unwrap();

    let id = causaloid.id();
    assert_eq!(id, 01);
}

#[test]
fn test_get_root_index() {
    let mut g = CausaloidGraph::new();
    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    let r_index = g.get_root_index().unwrap();
    assert_eq!(root_index, r_index);
}

#[test]
fn test_add_causaloid() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);
}

#[test]
fn test_contains_causaloid() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);
}

#[test]
fn test_get_causaloid() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let causaloid = g.get_causaloid(index).unwrap();

    let id = causaloid.id();
    let description = causaloid.description();

    assert_eq!(id, 01);
    assert_eq!(description, "tests whether data exceeds threshold of 0.55");
}

#[test]
fn test_remove_causaloid() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let causaloid = g.get_causaloid(index).unwrap();

    let id = causaloid.id();
    let description = causaloid.description();

    assert_eq!(id, 01);
    assert_eq!(description, "tests whether data exceeds threshold of 0.55");

    let res = g.remove_causaloid(index);
    assert!(res.is_ok());

    let contains = g.contains_causaloid(index);
    assert!(!contains);
}

#[test]
fn test_add_edge() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let res = g.add_edge(idx_a, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(contains_edge);
}

#[test]
fn test_add_edg_with_weight() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
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
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
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
fn test_all_true() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let all_true = g.all_active();
    assert!(!all_true);

    let obs = 0.99;
    let res = g.reason_single_cause(idx_a, &[obs]).unwrap();
    assert!(res);

    let all_true = g.all_active();
    assert_eq!(all_true, all_true);
}

#[test]
fn test_number_active() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    let obs = 0.99;
    let res = g.reason_single_cause(idx_a, &[obs]).unwrap();
    assert!(res);

    let number_active = g.number_active();
    assert_eq!(number_active, 1.0);
}


#[test]
fn test_percent_active() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let obs = 0.99;
    let res = g.reason_single_cause(idx_a, &[obs]).unwrap();
    assert!(res);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 50.0);

    let obs = 0.99;
    let res = g.reason_single_cause(idx_b, &[obs]).unwrap();
    assert!(res);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);
}

#[test]
fn test_size() {
    let mut g = CausaloidGraph::new();

    let size = g.size();
    assert_eq!(size, 0);

    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let size = g.size();
    assert_eq!(size, 2);
}

#[test]
fn test_is_empty() {
    let mut g = CausaloidGraph::new();
    let is_empty = g.is_empty();
    assert!(is_empty);

    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let is_empty = g.is_empty();
    assert!(!is_empty);
}

#[test]
fn test_clear() {
    let mut g = CausaloidGraph::new();
    let is_empty = g.is_empty();
    assert!(is_empty);

    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let is_empty = g.is_empty();
    assert!(!is_empty);

    g.clear();
    let is_empty = g.is_empty();
    assert!(is_empty);
}

#[test]
fn test_count_edges() {
    let mut g = CausaloidGraph::new();
    let count_edges = g.number_edges();
    assert_eq!(count_edges, 0);

    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let res = g.add_edge(idx_a, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(contains_edge);

    let count_edges = g.number_edges();
    assert_eq!(count_edges, 1);

    let res = g.remove_edge(idx_a, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(!contains_edge);

    let count_edges = g.number_edges();
    assert_eq!(count_edges, 0);
}

#[test]
fn test_count_nodes() {
    let mut g = CausaloidGraph::new();
    let count_nodes = g.number_nodes();
    assert_eq!(count_nodes, 0);

    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let count_nodes = g.number_nodes();
    assert_eq!(count_nodes, 1);

    let causaloid = g.get_causaloid(index).unwrap();

    let id = causaloid.id();
    assert_eq!(id, 01);

    let res = g.remove_causaloid(index);
    assert!(res.is_ok());


    let contains = g.contains_causaloid(index);
    assert!(!contains);

    let count_nodes = g.number_nodes();
    assert_eq!(count_nodes, 0);
}

#[test]
fn test_get_graph() {
    let g: CausaloidGraph<Causaloid<Dataoid, Spaceoid, Tempoid, SpaceTempoid>> = CausaloidGraph::new();

    let size = g.size();
    assert_eq!(size, 0);

    let graph = g.get_graph();
    assert_eq!(graph.edge_count(), 0);
    assert_eq!(graph.node_count(), 0);
}