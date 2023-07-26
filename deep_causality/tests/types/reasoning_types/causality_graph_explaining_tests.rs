// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use deep_causality::prelude::NodeIndex;
use deep_causality::protocols::causable_graph::{CausableGraph, CausableGraphReasoning};
use deep_causality::utils::bench_utils_graph;


#[test]
fn test_explain_all_causes() {
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

    // Full reasoning over the entire graph
    //
    let all_true = g.all_active();
    assert!(!all_true);

    let res = g.reason_all_causes(&data, None).unwrap();
    assert!(res);

    // Verify that the graph is fully active.
    let all_active = g.all_active();
    assert!(all_active);

    let percent_active = g.percent_active();
    assert_eq!(percent_active, 100.0);

    let total_nodes = g.node_count() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Explain all full reasoning over the entire graph
    //
    let res = g.explain_all_causes().unwrap();
    let expected = format!("\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 on last data 0.99 evaluated to true\n\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 on last data 0.99 evaluated to true\n\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 on last data 0.99 evaluated to true\n");
    assert_eq!(res, expected);
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
    let (g, data) = bench_utils_graph::get_small_multi_cause_graph_and_data();

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

    let total_nodes = g.node_count() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Explain partial reasoning over sub-graph
    //
    let start_index = NodeIndex::new(2);
    let res = g.explain_subgraph_from_cause(start_index).unwrap();
    let expected = format!("\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 on last data 0.99 evaluated to true\n\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 on last data 0.99 evaluated to true\n");
    assert_eq!(res, expected);
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
    let (g, data) = bench_utils_graph::get_small_multi_cause_graph_and_data();

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

    let total_nodes = g.node_count() as f64;
    let number_active = g.number_active();
    assert_eq!(number_active, total_nodes);

    // Reasoning over shortest path through the graph
    //
    let start_index = NodeIndex::new(2);
    let stop_index = NodeIndex::new(3);
    let res = g.reason_shortest_path_between_causes(start_index, stop_index, &data, None).unwrap();
    assert!(res);

    // Explain partial reasoning over shortest path through the graph
    //
    let res = g.explain_shortest_path_between_causes(start_index, stop_index).unwrap();
    let expected = format!("\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 on last data 0.99 evaluated to true\n\n * Causaloid: 1 tests whether data exceeds threshold of 0.55 on last data 0.99 evaluated to true\n");
    assert_eq!(res, expected);
}