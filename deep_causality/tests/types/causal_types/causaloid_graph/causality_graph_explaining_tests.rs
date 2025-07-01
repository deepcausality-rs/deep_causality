/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

use deep_causality::utils_test::{test_utils, test_utils_graph};

#[test]
fn test_explain_all_causes() {
    // Reasons over a multi-cause graph:
    //  root(0)
    //  /  \
    //A(1) B(2)
    //  \ /
    //  C(3)
    // We assume two causes (A and B) for C and single cause for A and B.
    let (g, data) = test_utils_graph::get_small_multi_cause_graph_and_data();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    // Full reasoning over the entire graph
    //
    let all_true = g.all_active();
    assert!(!all_true);

    let res = g.reason_all_causes(&data, None);
    assert!(res.is_ok());

    // Verify that the graph is fully active.
    let all_active = g.all_active();
    assert!(all_active);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let total_nodes = g.number_nodes() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Explain all full reasoning over the entire graph
    //
    let res = g.explain_all_causes().unwrap();
    let expected = "\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 evaluated to true\n\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 evaluated to true\n\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 evaluated to true\n".to_string();
    assert_eq!(res, expected);
}

#[test]
fn test_explain_all_causes_error() {
    let mut g = CausaloidGraph::new();
    assert!(g.is_empty());

    // Error: Graph is empty
    let res = g.explain_all_causes();
    assert!(res.is_err());

    // Add causaloid A
    let causaloid = test_utils::get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);
    assert!(!g.is_empty());

    // Error: Graph does not contains root causaloid
    let res = g.explain_all_causes();
    assert!(res.is_err());
}

#[test]
fn test_explain_subgraph_from_cause() {
    // Reasons over a multi-cause graph:
    //  root(0)
    //  /  \
    //A(1) B(2)
    //  \ /
    //  C(3)
    // We assume two causes (A and B) for C and single cause for A and B.
    let (g, data) = test_utils_graph::get_small_multi_cause_graph_and_data();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    // Full reasoning over the entire graph
    //
    let all_true = g.all_active();
    assert!(!all_true);

    let res = g
        .reason_all_causes(&data, None)
        .expect("Failed to reason over the entire graph");
    assert!(res);

    // Verify that the graph is fully active.
    let all_true = g.all_active();
    assert!(all_true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let total_nodes = g.number_nodes() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Explain partial reasoning over sub-graph
    //
    let start_index = 2;
    let res = g.explain_subgraph_from_cause(start_index).unwrap();
    let expected = "\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 evaluated to true\n\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 evaluated to true\n".to_string();
    assert_eq!(res, expected);
}

#[test]
fn test_explain_subgraph_from_cause_error() {
    let mut g = CausaloidGraph::new();
    assert!(g.is_empty());

    let no_idx = 99;

    // Error: Graph is empty
    let res = g.explain_subgraph_from_cause(no_idx);
    assert!(res.is_err());

    // Add causaloid A
    let causaloid = test_utils::get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);
    assert!(!g.is_empty());

    // Error: No path
    let res = g.explain_subgraph_from_cause(idx_a);
    assert!(res.is_err());
}

#[test]
fn test_explain_shortest_path_between_causes() {
    // Reasons over a multi-cause graph:
    //  root(0)
    //  /  \
    //A(1) B(2)
    //  \ /
    //  C(3)
    // We assume two causes (A and B) for C and single cause for A and B.
    let (g, data) = test_utils_graph::get_small_multi_cause_graph_and_data();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    // Full reasoning over the entire graph
    //
    let all_true = g.all_active();
    assert!(!all_true);

    let res = g.reason_all_causes(&data, None).expect("`");
    assert!(res);

    // Verify that the graph is fully active.
    let all_true = g.all_active();
    assert!(all_true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let total_nodes = g.number_nodes() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Reasoning over shortest path through the graph
    //
    let start_index = 2;
    let stop_index = 3;
    let res = g
        .reason_shortest_path_between_causes(start_index, stop_index, &data, None)
        .unwrap();
    assert!(res);

    // Explain partial reasoning over shortest path through the graph
    //
    let res = g
        .explain_shortest_path_between_causes(start_index, stop_index)
        .unwrap();
    let expected = "\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 evaluated to true\n\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 evaluated to true\n";
    assert_eq!(res, expected);
}
