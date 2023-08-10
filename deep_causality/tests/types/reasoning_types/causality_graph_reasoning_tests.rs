// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::*;
use deep_causality::utils::bench_utils_graph;

#[test]
fn test_linear_graph() {
    // Reasons over a linear graph:
    // root(0) -> A(1) -> B(2) -> C(3) ... XYZ(100)
    // We assume a linear chain of causality.
    let (g, data) = bench_utils_graph::get_small_linear_graph_and_data();

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
    let res = g.reason_all_causes(&data, None).unwrap();
    assert!(res);

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
    let (g, data) = bench_utils_graph::get_small_multi_cause_graph_and_data();

    // Verify that the graph is fully inactive.
    let percent_active = g.percent_active();
    assert_eq!(percent_active, 0.0);

    let number_active = g.number_active();
    assert_eq!(number_active, 0.0);

    // Single reasoning
    let obs = 0.99;
    let index = 2;
    let res = g.reason_single_cause(index, &[obs]).unwrap();
    assert!(res);

    let total_nodes = 1.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning from B (index 2)
    let index = 2;
    let res = g.reason_subgraph_from_cause(index, &data, None).unwrap();
    assert!(res);

    let total_nodes = 2.0;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Single reasoning
    // Only reason over C (index 3)
    let obs = 0.02;
    let index = 3;
    let res = g.reason_single_cause(index, &[obs]).unwrap();

    // we expect the result to be false because the
    // observation of 0.02 is well below the threshold
    // and thus node is not active anymore.
    assert!(!res);

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

    let res = g.reason_all_causes(&data, None).unwrap();
    assert!(res);

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
    let (g, data) = bench_utils_graph::get_small_multi_layer_cause_graph_and_data();

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
    let res = g.reason_single_cause(index, &[obs]).unwrap();
    assert!(res);

    let total_nodes = 1.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning
    // Start at C, and reason over C, F, G
    let index = 3;
    let res = g.reason_subgraph_from_cause(index, &data, None).unwrap();
    assert!(res);

    // We expect 3 active nodes because C,F, and G
    // must be active after reasoning.
    let total_nodes = 3.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning
    // Start at B, and reason over B , E, and F
    let index = 2;
    let res = g.reason_subgraph_from_cause(index, &data, None).unwrap();
    assert!(res);

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
    let res = g.reason_single_cause(index, &[obs]).unwrap();

    // we expect the result to be false because the
    // observation of 0.02 is well below the threshold
    // and thus node is not active anymore.
    assert!(!res);

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
    let res = g.reason_all_causes(&data, None).unwrap();
    assert!(res);

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
    let (g, data) = bench_utils_graph::get_left_imbalanced_cause_graph();

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
    let res = g.reason_single_cause(index, &[obs]).unwrap();
    assert!(res);

    let total_nodes = 1.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning
    // Start at A, and reason over A, D, E
    let index = 1;
    let res = g.reason_subgraph_from_cause(index, &data, None).unwrap();
    assert!(res);

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
    let res = g.reason_shortest_path_between_causes(start_index, stop_index, &data, None).unwrap();
    assert!(res);

    // We expect 4 active nodes because node A and D were already active before
    let total_nodes = 4.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Single reasoning
    // Only reason over A (index 1)
    let obs = 0.02;
    let index = 1;
    let res = g.reason_single_cause(index, &[obs]).unwrap();

    // we expect the result to be false because the
    // observation of 0.02 is well below the threshold
    // and thus node is not active anymore.
    assert!(!res);

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
    let res = g.reason_all_causes(&data, None).unwrap();
    assert!(res);

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
fn test_right_imbalanced_cause_graph() {
    //   root(0)
    //  /   |   \
    // A(1) B(2) C(3)
    //           /  \
    //         D(4) E(5)
    // We assume single causality with an imbalance to the right side of the graph.
    let (g, data) = bench_utils_graph::get_right_imbalanced_cause_graph();

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
    let res = g.reason_single_cause(index, &[obs]).unwrap();
    assert!(res);

    let total_nodes = 1.0_f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Partial reasoning
    // Start at C, and reason over C, F, G
    let index = 3;
    let res = g.reason_subgraph_from_cause(index, &data, None).unwrap();
    assert!(res);

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
    let res = g.reason_single_cause(index, &[obs]).unwrap();

    // we expect the result to be false because the
    // observation of 0.02 is well below the threshold
    // and thus the node remains inactive.
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
    let res = g.reason_all_causes(&data, None).unwrap();
    assert!(res);

    // Verify that the graph is fully active.
    let all_true = g.all_active();
    assert!(all_true);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let total_nodes = g.number_nodes() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);
}