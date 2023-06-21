/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use deep_causality::prelude::*;
use deep_causality::utils::{bench_utils_graph, test_utils};

#[test]
fn test_add_root_causaloid() {
    let mut g = CausaloidGraph::new();
    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert_eq!(contains_root, true);
}

#[test]
fn test_get_root_causaloid() {
    let mut g = CausaloidGraph::new();
    let root_causaloid = test_utils::get_test_causaloid();

    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert_eq!(contains_root, true);

    let causaloid = g.get_causaloid(root_index).unwrap();

    let id = causaloid.id();
    causaloid.description();
    let data_set_id = causaloid.data_set_id();

    assert_eq!(id, 01);
    assert_eq!(data_set_id, "Test data");
}

#[test]
fn test_add_causaloid() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert_eq!(contains, true);
}

#[test]
fn test_contains_causaloid() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert_eq!(contains, true);
}

#[test]
fn test_get_causaloid() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert_eq!(contains, true);

    let causaloid = g.get_causaloid(index).unwrap();

    let id = causaloid.id();
    let description = causaloid.description();
    let data_set_id = causaloid.data_set_id();

    assert_eq!(id, 01);
    assert_eq!(description, "tests whether data exceeds threshold of 0.55");
    assert_eq!(data_set_id, "Test data");
}

#[test]
fn test_remove_causaloid() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert_eq!(contains, true);

    let causaloid = g.get_causaloid(index).unwrap();

    let id = causaloid.id();
    let description = causaloid.description();
    let data_set_id = causaloid.data_set_id();

    assert_eq!(id, 01);
    assert_eq!(description, "tests whether data exceeds threshold of 0.55");
    assert_eq!(data_set_id, "Test data");

    g.remove_causaloid(index);
    let contains = g.contains_causaloid(index);
    assert_eq!(contains, false);
}

#[test]
fn test_add_edge() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    g.add_edge(idx_a, idx_b);
    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert_eq!(contains_edge, true);
}

#[test]
fn test_remove_edge() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    g.add_edge(idx_a, idx_b);
    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert_eq!(contains_edge, true);

    g.remove_edge(idx_a, idx_b);
    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert_eq!(contains_edge, false);
}

#[test]
fn test_all_true() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    let all_true = g.all_active();
    assert_eq!(all_true, false);

    let obs = 0.99;
    let res = g.reason_single_cause(idx_a, &[obs]).unwrap();
    assert_eq!(res, true);

    let all_true = g.all_active();
    assert_eq!(all_true, all_true);
}

#[test]
fn test_number_active() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    let obs = 0.99;
    let res = g.reason_single_cause(idx_a, &[obs]).unwrap();
    assert_eq!(res, true);

    let number_active = g.number_active();
    assert_eq!(number_active, 1.0);
}


#[test]
fn test_percent_active() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let obs = 0.99;
    let res = g.reason_single_cause(idx_a, &[obs]).unwrap();
    assert_eq!(res, true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 50.0);

    let obs = 0.99;
    let res = g.reason_single_cause(idx_b, &[obs]).unwrap();
    assert_eq!(res, true);

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
    assert_eq!(contains_a, true);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    let size = g.size();
    assert_eq!(size, 2);
}

#[test]
fn test_is_empty() {
    let mut g = CausaloidGraph::new();
    let is_empty = g.is_empty();
    assert_eq!(is_empty, true);

    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    let is_empty = g.is_empty();
    assert_eq!(is_empty, false);
}

#[test]
fn test_clear() {
    let mut g = CausaloidGraph::new();
    let is_empty = g.is_empty();
    assert_eq!(is_empty, true);

    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    let is_empty = g.is_empty();
    assert_eq!(is_empty, false);

    g.clear();
    let is_empty = g.is_empty();
    assert_eq!(is_empty, true);
}

#[test]
fn test_count_edges() {
    let mut g = CausaloidGraph::new();
    let count_edges = g.count_edges();
    assert_eq!(count_edges, 0);

    let causaloid = test_utils::get_test_causaloid();

    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    g.add_edge(idx_a, idx_b);
    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert_eq!(contains_edge, true);

    let count_edges = g.count_edges();
    assert_eq!(count_edges, 1);

    g.remove_edge(idx_a, idx_b);
    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert_eq!(contains_edge, false);

    let count_edges = g.count_edges();
    assert_eq!(count_edges, 0);
}

#[test]
fn test_count_nodes() {
    let mut g = CausaloidGraph::new();
    let count_nodes = g.count_nodes();
    assert_eq!(count_nodes, 0);

    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert_eq!(contains, true);

    let count_nodes = g.count_nodes();
    assert_eq!(count_nodes, 1);

    let causaloid = g.get_causaloid(index).unwrap();

    let id = causaloid.id();
    let description = causaloid.description();
    let data_set_id = causaloid.data_set_id();

    assert_eq!(id, 01);
    assert_eq!(description, "tests whether data exceeds threshold of 0.55");
    assert_eq!(data_set_id, "Test data");

    g.remove_causaloid(index);
    let contains = g.contains_causaloid(index);
    assert_eq!(contains, false);

    let count_nodes = g.count_nodes();
    assert_eq!(count_nodes, 0);
}


#[test]
fn test_reason_all_causes() {
    let mut g = CausaloidGraph::new();

    // Add root causaloid
    let root_causaloid = test_utils::get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert_eq!(contains_root, true);

    // Add causaloid A
    let causaloid = test_utils::get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a);
    let contains_edge = g.contains_edge(root_index, idx_a);
    assert_eq!(contains_edge, true);

    // Add causaloid B
    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b);
    let contains_edge = g.contains_edge(root_index, idx_b);
    assert_eq!(contains_edge, true);

    // Add causaloid C
    let causaloid = test_utils::get_test_causaloid();
    let idx_c = g.add_causaloid(causaloid);
    let contains_c = g.contains_causaloid(idx_c);
    assert_eq!(contains_c, true);

    // Link causaloid C to A
    g.add_edge(idx_a, idx_c);
    let contains_edge = g.contains_edge(idx_a, idx_c);
    assert_eq!(contains_edge, true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active() as f64;
    assert_eq!(number_active, 0.0);

    let all_true = g.all_active();
    assert_eq!(all_true, false);

    let data = [0.99, 0.98, 0.97];
    let res = g.reason_all_causes(&data, None).unwrap();
    assert_eq!(res, true);

    let all_true = g.all_active();
    assert_eq!(all_true, true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let number_active = g.number_active() as f64;
    assert_eq!(number_active, 4.0);
}

#[test]
fn test_reason_subgraph_from_cause() {
    let mut g = CausaloidGraph::new();

    // Add root causaloid
    let root_causaloid = test_utils::get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert_eq!(contains_root, true);

    // Add causaloid A
    let causaloid = test_utils::get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert_eq!(contains_a, true);

    // Link causaloid A to root causaloid
    g.add_edge(root_index, idx_a);
    let contains_edge = g.contains_edge(root_index, idx_a);
    assert_eq!(contains_edge, true);

    // Add causaloid B
    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert_eq!(contains_b, true);

    // Link causaloid B to root causaloid
    g.add_edge(root_index, idx_b);
    let contains_edge = g.contains_edge(root_index, idx_b);
    assert_eq!(contains_edge, true);

    // Add causaloid C
    let causaloid = test_utils::get_test_causaloid();
    let idx_c = g.add_causaloid(causaloid);
    let contains_c = g.contains_causaloid(idx_c);
    assert_eq!(contains_c, true);

    // Link causaloid C to A
    g.add_edge(idx_a, idx_c);
    let contains_edge = g.contains_edge(idx_a, idx_c);
    assert_eq!(contains_edge, true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let all_true = g.all_active();
    assert_eq!(all_true, false);

    let data = [0.99, 0.98];
    let res = g.reason_subgraph_from_cause(idx_a, &data, None).unwrap();
    assert_eq!(res, true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 50.0);
}

#[test]
fn test_reason_shortest_path_between_causes() {
    // Reasons over a linear graph:
    // root(0) -> A(1) -> B(2) -> C(3) ... XYZ(100)
    // We assume a linear chain of causality.
    let (g, data) = bench_utils_graph::get_small_linear_graph_and_data();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active() as f64;
    assert_eq!(number_active, 0.0);

    let all_true = g.all_active();
    assert_eq!(all_true, false);

    let start_index = NodeIndex::new(10);
    let stop_index = NodeIndex::new(19);
    let res = g.reason_shortest_path_between_causes(
        start_index,
        stop_index,
        &data,
        None,
    ).unwrap();
    assert_eq!(res, true);

    let all_true = g.all_active();
    assert_eq!(all_true, false);

    let number_true = g.number_active();
    assert_eq!(number_true, 10.0);

    let start_index = NodeIndex::new(30);
    let stop_index = NodeIndex::new(49);
    let res = g.reason_shortest_path_between_causes(
        start_index,
        stop_index,
        &data,
        None,
    ).unwrap();
    assert_eq!(res, true);

    let number_true = g.number_active();
    assert_eq!(number_true, 30.0);

    let all_true = g.all_active();
    assert_eq!(all_true, false);
}

#[test]
fn test_reason_single_cause() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert_eq!(contains, true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let all_true = g.all_active();
    assert_eq!(all_true, false);

    let obs = 0.99;
    let res = g.reason_single_cause(index, &[obs]).unwrap();

    assert_eq!(res, true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let all_true = g.all_active();
    assert_eq!(all_true, true);
}