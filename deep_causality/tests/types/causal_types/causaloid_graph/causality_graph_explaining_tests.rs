/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

use deep_causality::utils_test::{test_utils, test_utils_graph};

#[test]
fn test_explain_all_causes() {
    // Graph: root(0) -> A(1), root(0) -> B(2), A(1) -> C(3), B(2) -> C(3)
    let (g, _data) = test_utils_graph::get_small_multi_cause_graph_and_data();
    let root_index = g.get_root_index().unwrap();

    // Before evaluation, explain returns a message that no nodes have been evaluated.
    let pre_explanation = g.explain_all_causes().unwrap();
    assert_eq!(
        pre_explanation,
        "No nodes in the graph have been evaluated or produced an explainable effect."
    );

    // Evaluate the entire graph from the root.
    let effect = PropagatingEffect::Numerical(0.99);
    let res = g.evaluate_subgraph_from_cause(root_index, &effect);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // Explain the fully evaluated graph.
    // The explanation order is determined by a Breadth-First Search from the root.
    let explanation = g.explain_all_causes().unwrap();
    let base_expl = "Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: PropagatingEffect::Deterministic(true)";
    // Expected order from BFS: 0, 1, 2, 3
    let expected = format!(" * {base_expl}\n * {base_expl}\n * {base_expl}\n * {base_expl}");
    assert_eq!(explanation, expected);
}

#[test]
fn test_explain_all_causes_error_conditions() {
    // Test with an empty graph
    let mut g: BaseCausalGraph = CausaloidGraph::new(0);
    assert!(g.is_empty());
    let res = g.explain_all_causes().unwrap();
    assert_eq!(res, "The causal graph is empty.");

    // Test with a graph that has no root
    let causaloid = test_utils::get_test_causaloid();
    g.add_causaloid(causaloid)
        .expect("Failed to add causaloid A");
    assert!(!g.is_empty());
    let res = g.explain_all_causes();
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "CausalityError: Cannot explain all causes: Graph has no root node."
    );
}

#[test]
fn test_explain_subgraph_from_cause() {
    // Graph: root(0) -> A(1), root(0) -> B(2), A(1) -> C(3), B(2) -> C(3)
    let (g, _data) = test_utils_graph::get_small_multi_cause_graph_and_data();
    let root_index = g.get_root_index().unwrap();

    // Evaluate the entire graph first.
    let effect = PropagatingEffect::Numerical(0.99);
    g.evaluate_subgraph_from_cause(root_index, &effect).unwrap();

    // Explain a subgraph starting from node 2.
    // The traversal will visit nodes 2 and its descendant, 3.
    let start_index = 2;
    let res = g.explain_subgraph_from_cause(start_index).unwrap();
    let base_expl = "Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: PropagatingEffect::Deterministic(true)";
    let expected = format!(" * {base_expl}\n * {base_expl}");
    assert_eq!(res, expected);
}

#[test]
fn test_explain_subgraph_from_cause_error_conditions() {
    let mut g: BaseCausalGraph = CausaloidGraph::new(0);
    assert!(g.is_empty());

    // Error: Graph is empty
    let res = g.explain_subgraph_from_cause(99);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "CausalityGraphError: Graph is empty"
    );

    // Error: Start node does not exist
    let causaloid = test_utils::get_test_causaloid();
    g.add_causaloid(causaloid).expect("Failed to add causaloid");
    g.freeze();
    let res = g.explain_subgraph_from_cause(99);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "CausalityGraphError: Graph does not contains start causaloid"
    );
}

#[test]
fn test_explain_shortest_path_between_causes() {
    // Graph: root(0) -> A(1), root(0) -> B(2), A(1) -> C(3), B(2) -> C(3)
    let (g, _data) = test_utils_graph::get_small_multi_cause_graph_and_data();
    let root_index = g.get_root_index().unwrap();

    // Evaluate the entire graph first.
    let effect = PropagatingEffect::Numerical(0.99);
    g.evaluate_subgraph_from_cause(root_index, &effect).unwrap();

    // Explain the shortest path from node 2 to node 3.
    // The path is just [2, 3].
    let start_index = 2;
    let stop_index = 3;
    let res = g
        .explain_shortest_path_between_causes(start_index, stop_index)
        .unwrap();

    let base_expl = "Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: PropagatingEffect::Deterministic(true)";
    let expected = format!(" * {base_expl}\n * {base_expl}");
    assert_eq!(res, expected);
}
