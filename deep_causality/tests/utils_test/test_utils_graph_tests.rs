/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::CausableGraph;
use deep_causality::utils_test::test_utils_graph::{
    build_left_imbalanced_cause_graph, build_linear_graph, build_multi_cause_graph,
    build_multi_layer_cause_graph, build_right_imbalanced_cause_graph, generate_sample_data,
    get_left_imbalanced_cause_graph, get_right_imbalanced_cause_graph,
    get_small_multi_cause_graph_and_data, get_small_multi_layer_cause_graph_and_data,
};

#[test]
fn test_generate_sample_data() {
    let data: [f64; 5] = generate_sample_data();
    assert_eq!(data.len(), 5);
    assert!(data.iter().all(|&x| x == 0.99));
}

#[test]
fn test_build_linear_graph() {
    let graph = build_linear_graph(5);
    assert!(graph.is_frozen());
    assert_eq!(graph.number_nodes(), 5);
}

#[test]
fn test_build_multi_cause_graph() {
    let graph = build_multi_cause_graph();
    assert!(graph.is_frozen());
    assert_eq!(graph.number_nodes(), 4); // root, A, B, C
    assert_eq!(graph.number_edges(), 4); // root->A, root->B, A->C, B->C
}

#[test]
fn test_get_small_multi_cause_graph_and_data() {
    let (graph, data) = get_small_multi_cause_graph_and_data();
    assert!(graph.is_frozen());
    assert_eq!(graph.number_nodes(), 4);
    assert_eq!(data.len(), 4);
}

#[test]
fn test_build_multi_layer_cause_graph() {
    let graph = build_multi_layer_cause_graph();
    assert!(graph.is_frozen());
    assert_eq!(graph.number_nodes(), 8); // root, A,B,C, D,E,F,G
    assert_eq!(graph.number_edges(), 9); // root->A, root->B, root->C, A->D, A->E, B->E, B->F, C->F, C->G
}

#[test]
fn test_get_small_multi_layer_cause_graph_and_data() {
    let (graph, data) = get_small_multi_layer_cause_graph_and_data();
    assert!(graph.is_frozen());
    assert_eq!(graph.number_nodes(), 8);
    assert_eq!(data.len(), 9); // 8 nodes + 1 for root
}

#[test]
fn test_build_left_imbalanced_cause_graph() {
    let graph = build_left_imbalanced_cause_graph();
    assert!(graph.is_frozen());
    assert_eq!(graph.number_nodes(), 6); // root, A,B,C, D,E
    assert_eq!(graph.number_edges(), 5); // root->A, root->B, root->C, A->D, A->E
}

#[test]
fn test_get_left_imbalanced_cause_graph() {
    let (graph, data) = get_left_imbalanced_cause_graph();
    assert!(graph.is_frozen());
    assert_eq!(graph.number_nodes(), 6);
    assert_eq!(data.len(), 7); // 6 nodes + 1 for root
}

#[test]
fn test_build_right_imbalanced_cause_graph() {
    let graph = build_right_imbalanced_cause_graph();
    assert!(graph.is_frozen());
    assert_eq!(graph.number_nodes(), 6); // root, A,B,C, D,E
    assert_eq!(graph.number_edges(), 5); // root->A, root->B, root->C, C->D, C->E
}

#[test]
fn test_get_right_imbalanced_cause_graph() {
    let (graph, data) = get_right_imbalanced_cause_graph();
    assert!(graph.is_frozen());
    assert_eq!(graph.number_nodes(), 6);
    assert_eq!(data.len(), 7); // 6 nodes + 1 for root
}
