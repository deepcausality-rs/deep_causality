/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage note: the `MonadicCausableGraphReasoning` BFS / path walks carry defensive arms that
//! re-fetch a causaloid by an index taken from the graph's own frozen adjacency, or handle an
//! `outbound_edges` failure. On a frozen, validated graph those cannot fail, so they are left
//! uncovered rather than forced through a corrupted graph (the same rationale as the stateful trait).

use deep_causality::utils_test::test_utils;
use deep_causality::*;

fn relay_to_2(_obs: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_effect(CausalEffect::relay_to(2, CausalEffect::value(true)))
}

fn relay_to_5(_obs: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_effect(CausalEffect::relay_to(5, CausalEffect::value(true)))
}

#[test]
fn test_evaluate_subgraph_requires_a_frozen_graph() {
    let mut g = CausaloidGraph::new(0);
    let c = test_utils::get_test_causaloid_deterministic_input_output();
    let idx = g.add_root_causaloid(c).expect("root");
    // Not frozen.
    let res = g.evaluate_subgraph_from_cause(idx, &PropagatingEffect::from_value(true));
    assert!(res.is_err());
    assert!(res.error().unwrap().to_string().contains("not frozen"));
}

#[test]
fn test_evaluate_subgraph_rejects_a_missing_start_index() {
    let mut g: BaseCausalGraph = CausaloidGraph::new(0);
    g.freeze();
    let res = g.evaluate_subgraph_from_cause(99, &PropagatingEffect::from_value(true));
    assert!(res.is_err());
    assert!(
        res.error()
            .unwrap()
            .to_string()
            .contains("does not contain")
    );
}

#[test]
fn test_evaluate_subgraph_short_circuits_on_a_node_error() {
    let mut g = CausaloidGraph::new(0);
    let root = test_utils::get_test_causaloid_deterministic_input_output();
    let i0 = g.add_root_causaloid(root).expect("root");
    let err_node = test_utils::get_test_error_causaloid();
    let i1 = g.add_causaloid(err_node).expect("err");
    g.add_edge(i0, i1).expect("edge");
    g.freeze();

    let res = g.evaluate_subgraph_from_cause(i0, &PropagatingEffect::from_value(true));
    assert!(res.is_err());
    assert!(res.error().unwrap().to_string().contains("Test error"));
}

#[test]
fn test_evaluate_subgraph_follows_a_relay_to_a_valid_target() {
    // 0 (relays to 2) -> 1 ; 2 stands alone. The relay jumps straight to node 2.
    let mut g = CausaloidGraph::new(0);
    let i0 = g
        .add_root_causaloid(Causaloid::new(0, relay_to_2, "relayer"))
        .expect("root");
    let i1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic_input_output())
        .expect("n1");
    let i2 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic_input_output())
        .expect("n2");
    g.add_edge(i0, i1).expect("edge");
    g.freeze();

    let res = g.evaluate_subgraph_from_cause(i0, &PropagatingEffect::from_value(true));
    assert!(res.is_ok(), "got {:?}", res.error());
    let _ = i2;
}

#[test]
fn test_evaluate_subgraph_rejects_a_relay_to_a_missing_target() {
    // The root relays to index 5, which the graph does not contain.
    let mut g = CausaloidGraph::new(0);
    let i0 = g
        .add_root_causaloid(Causaloid::new(0, relay_to_5, "relayer"))
        .expect("root");
    let i1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic_input_output())
        .expect("n1");
    g.add_edge(i0, i1).expect("edge");
    g.freeze();

    let res = g.evaluate_subgraph_from_cause(i0, &PropagatingEffect::from_value(true));
    assert!(res.is_err());
    assert!(res.error().unwrap().to_string().contains("RelayTo target"));
}

fn add_one(x: bool) -> PropagatingEffect<bool> {
    // Distinct, index-independent effect: always resolves to true.
    let _ = x;
    PropagatingEffect::from_value(true)
}
fn add_ten(x: bool) -> PropagatingEffect<bool> {
    // Distinct, index-independent effect: always resolves to false.
    let _ = x;
    PropagatingEffect::from_value(false)
}

#[test]
fn test_evaluate_subgraph_branching_tree_returns_highest_index_leaf() {
    // Characterization test pinning the ascending-index schedule for a branching tree.
    //
    // The tree is root(0) -> A(1) and root(0) -> B(2). A and B are independent sibling leaves with
    // different effects (A -> true, B -> false); there is no reconvergence. The engine schedules
    // ready nodes by ascending node index (`BTreeSet::pop_first`), so B(2) is the last node
    // processed and `evaluate_subgraph_from_cause` returns B's effect. The edges are added in
    // non-ascending index order (root -> B before root -> A) to show the schedule follows the node
    // index rather than edge-insertion or discovery order: the result is the same either way.
    let mut g: BaseCausalGraph = CausaloidGraph::new(0);
    let root = g
        .add_causaloid(Causaloid::new(0, add_one, "root"))
        .expect("root");
    let a = g.add_causaloid(Causaloid::new(1, add_one, "A")).expect("A");
    let b = g.add_causaloid(Causaloid::new(2, add_ten, "B")).expect("B");
    // Edges added with the higher-index child first.
    g.add_edge(root, b).expect("edge root->B");
    g.add_edge(root, a).expect("edge root->A");
    g.freeze();

    let res = g.evaluate_subgraph_from_cause(root, &PropagatingEffect::from_value(true));
    assert!(res.is_ok(), "got {:?}", res.error());
    // B(2) is the highest-index leaf, so its effect (false) is the returned last effect.
    assert_eq!(res.value(), Some(&false));
}

#[test]
fn test_evaluate_subgraph_from_cause() {
    let mut g = CausaloidGraph::new(0);

    // Add root causaloid
    let root_causaloid = test_utils::get_test_causaloid_deterministic_input_output();
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    assert!(g.contains_causaloid(root_index));

    // Add causaloid A
    let causaloid_a = test_utils::get_test_causaloid_deterministic_input_output();
    let idx_a = g
        .add_causaloid(causaloid_a)
        .expect("Failed to add causaloid A");

    // Link A to root
    g.add_edge(root_index, idx_a).expect("Failed to add edge");

    // Add causaloid B
    let causaloid_b = test_utils::get_test_causaloid_deterministic_input_output();
    let idx_b = g
        .add_causaloid(causaloid_b)
        .expect("Failed to add causaloid B");

    // Link A to B
    g.add_edge(idx_a, idx_b).expect("Failed to add edge");

    // Now, we have a graph like this:
    // root -> A -> B
    g.freeze();

    // 2. Evaluate a subgraph starting from node A. This should activate nodes A and B.
    let effect = PropagatingEffect::from_value(true);
    let res = g.evaluate_subgraph_from_cause(idx_a, &effect);
    dbg!(&res);
    assert!(res.is_ok());
    // A evaluates from Boolean true to Boolean false;
    // B evaluates from Boolean false to Boolean true;
    // Thus the final effect is Boolean(true)
    assert_eq!(res.value(), Some(&true));
}
