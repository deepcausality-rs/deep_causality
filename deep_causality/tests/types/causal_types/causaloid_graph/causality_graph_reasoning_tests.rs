// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::*;

use crate::utils::{test_utils, test_utils_graph};

#[test]
fn test_reason_all_causes() {
    let mut g = CausaloidGraph::new();

    // Add root causaloid
    let root_causaloid = test_utils::get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    // Add causaloid A
    let causaloid = test_utils::get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    // Link causaloid A to root causaloid
    let res = g.add_edge(root_index, idx_a);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(root_index, idx_a);
    assert!(contains_edge);

    // Add causaloid B
    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    // Link causaloid B to root causaloid
    let res = g.add_edge(root_index, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(root_index, idx_b);
    assert!(contains_edge);

    // Add causaloid C
    let causaloid = test_utils::get_test_causaloid();
    let idx_c = g.add_causaloid(causaloid);
    let contains_c = g.contains_causaloid(idx_c);
    assert!(contains_c);

    // Link causaloid C to A
    let res = g.add_edge(idx_a, idx_c);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_c);
    assert!(contains_edge);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    let all_true = g.all_active();
    assert!(!all_true);

    let data = [0.99, 0.98, 0.97];
    let res = g.reason_all_causes(&data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 4.0);

    let all_true = g.all_active();
    assert!(all_true);
}

#[test]
fn test_reason_all_causes_error() {
    // Need type annotation as nothing is added and thus no type inference.
    let g: CausaloidGraph<BaseCausaloid> = CausaloidGraph::new();
    // Double check no root node exists
    assert!(!g.contains_root_causaloid());
    assert!(g.is_empty());

    // No root causaloid
    let data = [0.99, 0.98, 0.97];
    let res = g.reason_all_causes(&data, None);
    assert!(res.is_err());

    // No data
    let data = [];
    let res = g.reason_all_causes(&data, None);
    assert!(res.is_err());
}

#[test]
fn test_reason_subgraph_from_cause() {
    let mut g = CausaloidGraph::new();

    // Add root causaloid
    let root_causaloid = test_utils::get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    // Add causaloid A
    let causaloid = test_utils::get_test_causaloid();
    let idx_a = g.add_causaloid(causaloid);
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    // Link causaloid A to root causaloid
    let res = g.add_edge(root_index, idx_a);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(root_index, idx_a);
    assert!(contains_edge);

    // Add causaloid B
    let causaloid = test_utils::get_test_causaloid();
    let idx_b = g.add_causaloid(causaloid);
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    // Link causaloid B to root causaloid
    let res = g.add_edge(root_index, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(root_index, idx_b);
    assert!(contains_edge);

    // Add causaloid C
    let causaloid = test_utils::get_test_causaloid();
    let idx_c = g.add_causaloid(causaloid);
    let contains_c = g.contains_causaloid(idx_c);
    assert!(contains_c);

    // Link causaloid C to A
    let res = g.add_edge(idx_a, idx_c);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_c);
    assert!(contains_edge);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let all_true = g.all_active();
    assert!(!all_true);

    let data = [0.99, 0.98];
    let res = g.reason_subgraph_from_cause(idx_a, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 50.0);
}

#[test]
fn test_reason_subgraph_from_cause_error() {
    // Need type annotation as nothing is added and thus no type inference.
    let mut g = CausaloidGraph::new();

    // Double check graph is empty
    assert!(!g.contains_root_causaloid());

    let data = [0.99, 0.98];
    let no_idx = 99;

    // Error: Graph is empty
    let res = g.reason_subgraph_from_cause(no_idx, &data, None);
    assert!(res.is_err());

    // Add root causaloid
    let root_causaloid = test_utils::get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid);
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    let data = [];
    // Error: No data
    let res = g.reason_subgraph_from_cause(root_index, &data, None);
    assert!(res.is_err());
}

#[test]
fn test_reason_shortest_path_between_causes() {
    // Reasons over a linear graph:
    // root(0) -> A(1) -> B(2) -> C(3) ... XYZ(10)
    // We assume a linear chain of causality.
    let (g, data) = test_utils_graph::get_small_linear_graph_and_data();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    let all_true = g.all_active();
    assert!(!all_true);

    // the graph has 11 nodes, root plus 1 ...10
    // here we evaluate the first half, 1...6

    let start_index = 1;
    let stop_index = 6;
    let res = g.reason_shortest_path_between_causes(start_index, stop_index, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let all_true = g.all_active();
    assert!(!all_true);

    let number_true = g.number_active();
    assert_eq!(number_true, 6.0);

    // And then evaluate the second half, 7...9.

    let start_index = 7;
    let stop_index = 9;
    let res = g.reason_shortest_path_between_causes(start_index, stop_index, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // Note, because root node was not evaluated,
    // only 9 out of 10 nodes or 90% are active.
    let number_true = g.number_active();
    assert_eq!(number_true, 9.0);

    let percent_true = g.percent_active();
    assert_eq!(percent_true, 90.0);

    let all_true = g.all_active();
    assert!(!all_true);

    // Evaluate root node using shortest path between root and node 1
    let start_index = 0;
    let stop_index = 1;
    let res = g.reason_shortest_path_between_causes(start_index, stop_index, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // Now, all ten nodes or 100% are active.
    let number_true = g.number_active();
    assert_eq!(number_true, 10.0);

    let percent_true = g.percent_active();
    assert_eq!(percent_true, 100.0);

    let all_true = g.all_active();
    assert!(all_true);
}

#[test]
fn test_reason_shortest_path_between_causes_error() {
    let mut g = CausaloidGraph::new();
    assert!(g.is_empty());

    let obs = 0.99;

    let start_idx = 21;
    let stop_idx = 99;
    // Error: Graph is empty
    let res = g.reason_shortest_path_between_causes(start_idx, stop_idx, &[obs], None);
    assert!(res.is_err());

    let causaloid = test_utils::get_test_causaloid();
    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);

    // Graph is non-empty
    let start_idx = 21;
    let stop_idx = 99;
    // Error: Graph does not contains start causaloid
    let res = g.reason_shortest_path_between_causes(start_idx, stop_idx, &[obs], None);
    assert!(res.is_err());

    let start_idx = index;
    let stop_idx = 99;
    // Error: Graph does not contains stop causaloid
    let res = g.reason_shortest_path_between_causes(start_idx, stop_idx, &[obs], None);
    assert!(res.is_err());

    // Error: Start and Stop node identical
    let res = g.reason_shortest_path_between_causes(start_idx, start_idx, &[obs], None);
    assert!(res.is_err());

    let err_causaloid = test_utils::get_test_error_causaloid();
    let err_index = g.add_causaloid(err_causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let start_idx = index;

    // Error: No path found
    let res = g.reason_shortest_path_between_causes(start_idx, err_index, &[obs], None);
    assert!(res.is_err());
}

#[test]
fn test_reason_single_cause_single_data() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let all_true = g.all_active();
    assert!(!all_true);

    let obs = 0.99;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let all_true = g.all_active();
    assert!(all_true);
}

#[test]
fn test_reason_single_cause_multi_data() {
    let mut g = CausaloidGraph::new();
    let causaloid = test_utils::get_test_causaloid();

    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let all_true = g.all_active();
    assert!(!all_true);

    let res = g.reason_single_cause(index, &[0.99, 0.98, 0.97]);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let all_true = g.all_active();
    assert!(all_true);
}

#[test]
fn test_reason_single_cause_err_empty_graph() {
    let mut g = CausaloidGraph::new();
    assert!(g.is_empty());

    let index = 99;
    let obs = 0.99;

    // Error: Graph is empty
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_err());

    let causaloid = test_utils::get_test_causaloid();
    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);

    // Error Data are empty
    let res = g.reason_single_cause(index, &[]);
    assert!(res.is_err());

    // Error: Graph does not contains start causaloid
    let res = g.reason_single_cause(99, &[obs]);
    assert!(res.is_err());

    let causaloid = test_utils::get_test_error_causaloid();
    let index = g.add_causaloid(causaloid);
    let contains = g.contains_causaloid(index);
    assert!(contains);

    // CausalityGraphError: Test error
    let obs = 0.99;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_err());
}

#[test]
fn test_linear_graph() {
    // Reasons over a linear graph:
    // root(0) -> A(1) -> B(2) -> C(3) ... XYZ(100)
    // We assume a linear chain of causality.
    let (g, data) = test_utils_graph::get_small_linear_graph_and_data();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    let all_true = g.all_active();
    assert!(!all_true);

    // Full reasoning over the entire graph
    //
    // Note, the synthetic dataset is designed to activate all nodes.
    // In practice, we may not expect all nodes to be active
    // after full reasoning. Rather, only a certain number / percentage.
    // After the reasoning process, it's most sensible to use a percentage
    // of active nodes as threshold to decide whether to proceed further.
    let res = g.reason_all_causes(&data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // Verify that the graph is fully active.
    let all_true = g.all_active();
    assert!(all_true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let total_nodes = g.number_nodes() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);
}

#[test]
fn test_multi_cause_graph() {
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

    // Single reasoning
    let obs = 0.99;
    let index = 2;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let total_nodes = 1.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning from B (index 2)
    let index = 2;
    let res = g.reason_subgraph_from_cause(index, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let total_nodes = 2.0;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Single reasoning
    // Only reason over C (index 3)
    let obs = 0.02;
    let index = 3;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_ok());

    // we expect the result to be false because the
    // observation of 0.02 is well below the threshold
    // and thus node is not active anymore.
    assert!(!res.unwrap());

    // We expect one less node active because C was deactivated.
    // Hence only 1 active node.
    let total_nodes = 1.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Full reasoning over the entire graph
    //
    // Note, the synthetic dataset is designed to activate all nodes.
    // In practice, we may not expect all nodes to be active
    // after full reasoning. Rather, only a certain number / percentage.
    // After the reasoning process, it's most sensible to use a percentage
    // of active nodes as threshold to decide whether to proceed further.
    let all_true = g.all_active();
    assert!(!all_true);

    let res = g.reason_all_causes(&data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // Verify that the graph is fully active.
    let all_true = g.all_active();
    assert!(all_true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let total_nodes = g.number_nodes() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);
}

#[test]
fn test_multi_layer_cause_graph() {
    // Reasons over a multi-layer cause graph:
    //   root(0)
    //  /   |   \
    //A(1) B(2) C(3)
    // /  \  /  \ / \
    //D(4) E(5) F(6) G(7)
    // We assume multiple causes for each layer below the root node.
    let (g, data) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    let all_true = g.all_active();
    assert!(!all_true);

    // Single reasoning
    // Only reason over C
    let obs = 0.99;
    let index = 3;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let total_nodes = 1.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning
    // Start at C, and reason over C, F, G
    let index = 3;
    let res = g.reason_subgraph_from_cause(index, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // We expect 3 active nodes because C,F, and G
    // must be active after reasoning.
    let total_nodes = 3.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning
    // Start at B, and reason over B , E, and F
    let index = 2;
    let res = g.reason_subgraph_from_cause(index, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // We expect 2 active nodes because F was already activated
    // during the previous reasoning so only B and E will be active in
    // addition to the 3 nodes already activated.
    let total_nodes = 5.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Single reasoning
    // Only reason over G (index 7)
    let obs = 0.02;
    let index = 7;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_ok());

    // we expect the result to be false because the
    // observation of 0.02 is well below the threshold
    // and thus node is not active anymore.
    assert!(!res.unwrap());

    // We expect one less node active because G was deactivated.
    // Hence only 4 active nodes.
    let total_nodes = 4.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Full reasoning over the entire graph
    //
    // Note, the synthetic dataset is designed to activate all nodes.
    // In practice, we may not expect all nodes to be active
    // after full reasoning. Rather, only a certain number / percentage.
    // After the reasoning process, it's most sensible to use a percentage
    // of active nodes as threshold to decide whether to proceed further.
    let res = g.reason_all_causes(&data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // Verify that the graph is fully active.
    let total_nodes = g.number_nodes() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    let all_true = g.all_active();
    assert!(all_true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);
}

#[test]
fn test_left_imbalanced_cause_graph() {
    //   root(0)
    //  /   |   \
    // A(1) B(2) C(3)
    // /  \
    //D(4) E(5)
    // We assume single causality with an imbalance to the left side of the graph.
    let (g, data) = test_utils_graph::get_left_imbalanced_cause_graph();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    let all_true = g.all_active();
    assert!(!all_true);

    // Single reasoning
    // Only reason over C
    let obs = 0.99;
    let index = 3;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let total_nodes = 1.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning
    // Start at A, and reason over A, D, E
    let index = 1;
    let res = g.reason_subgraph_from_cause(index, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // We expect 4 active nodes because
    // single reasoning activated C
    // and partial reasoning activated A,D,and E, thus 4 in total
    let total_nodes = 4.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Selective sub-graph reasoning
    // Start at A, and stop at D
    let start_index = 1;
    let stop_index = 4;
    let res = g.reason_shortest_path_between_causes(start_index, stop_index, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // We expect 4 active nodes because node A and D were already active before
    let total_nodes = 4.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Single reasoning
    // Only reason over A (index 1)
    let obs = 0.02;
    let index = 1;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_ok());

    // we expect the result to be false because the
    // observation of 0.02 is well below the threshold
    // and thus node is not active anymore.
    assert!(!res.unwrap());

    // We expect one less node active because A was deactivated.
    // Hence only 3 active nodes.
    let total_nodes = 3.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Full reasoning over the entire graph
    //
    // Note, the synthetic dataset is designed to activate all nodes.
    // In practice, we may not expect all nodes to be active
    // after full reasoning. Rather, only a certain number / percentage.
    // After the reasoning process, it's most sensible to use a percentage
    // of active nodes as threshold to decide whether to proceed further.
    let res = g.reason_all_causes(&data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // Verify that the graph is fully active.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let all_true = g.all_active();
    assert!(all_true);

    let total_nodes = g.number_nodes() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);
}

#[test]
fn test_right_imbalanced_cause_graph() {
    //   root(0)
    //  /   |   \
    // A(1) B(2) C(3)
    //           /  \
    //         D(4) E(5)
    // We assume single causality with an imbalance to the right side of the graph.
    let (g, data) = test_utils_graph::get_right_imbalanced_cause_graph();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    let all_true = g.all_active();
    assert!(!all_true);

    // Single reasoning
    // Only reason over C
    let obs = 0.99;
    let index = 3;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let total_nodes = 1.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning
    // Start at C, and reason over C, F, G
    let index = 3;
    let res = g.reason_subgraph_from_cause(index, &data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // We expect 3 active nodes because
    // single reasoning activated C
    // and partial reasoning activated C, D, and , E
    // with C already active thus 3 in total.
    let total_nodes = 3.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Single reasoning
    // Only reason over C (index 2)
    let obs = 0.02;
    let index = 2;
    let res = g.reason_single_cause(index, &[obs]);
    assert!(res.is_ok());

    // we expect the result to be false because the
    // observation of 0.02 is well below the threshold
    // and thus the node remains inactive.
    let res = res.unwrap();
    assert!(!res);

    // We expect the same number of nodes as before
    // because B was not active before and the previous
    // single reasoning did not activated C hence 3 active nodes.
    let total_nodes = 3.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Full reasoning over the entire graph
    //
    // Note, the synthetic dataset is designed to activate all nodes.
    // In practice, we may not expect all nodes to be active
    // after full reasoning. Rather, only a certain number / percentage.
    // After the reasoning process, it's most sensible to use a percentage
    // of active nodes as threshold to decide whether to proceed further.
    let res = g.reason_all_causes(&data, None);
    assert!(res.is_ok());
    assert!(res.unwrap());

    // Verify that the graph is fully active.
    let all_true = g.all_active();
    assert!(all_true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let total_nodes = g.number_nodes() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);
}
