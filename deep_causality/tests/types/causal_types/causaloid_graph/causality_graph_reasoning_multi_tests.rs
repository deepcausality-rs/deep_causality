/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

use deep_causality::utils_test::test_utils_graph;

#[test]
fn test_multi_cause_graph() {
    // Reasons over a multi-cause graph:
    //  root(0)
    //  /  \
    //A(1) B(2)
    //  \ /
    //  C(3)
    // We assume two causes (A and B) for C and single cause for A and B.
    let (g, _data) = test_utils_graph::get_small_multi_cause_graph_and_data();
    let evidence = Evidence::Numerical(0.99);

    // 1. Verify that the graph is fully inactive before any evaluation.
    // State-checking methods return Err before any evaluation.
    assert!(g.percent_active().is_err());
    assert!(g.number_active().is_err());
    assert!(g.all_active().is_err());

    // 2. Single reasoning: Activate node B (index 2).
    let res = g.evaluate_single_cause(2, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // Now, only node B is active. Use the lenient counter for partially evaluated graphs.
    assert_eq!(g.count_known_active(), 1.0);

    // 3. Partial reasoning from B (index 2). This will also activate its descendant C (index 3).
    let res = g.evaluate_subgraph_from_cause(2, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // Now, nodes B and C are active.
    assert_eq!(g.count_known_active(), 2.0);

    // 4. Single reasoning: Deactivate node C (index 3).
    let deactivating_evidence = Evidence::Numerical(0.02);
    let res = g.evaluate_single_cause(3, &deactivating_evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(false));
    // Now, only node B remains active.
    assert_eq!(g.count_known_active(), 1.0);

    // 5. Full reasoning over the entire graph from the root.
    let res = g.evaluate(&evidence);
    assert!(res.is_ok());
    // The graph's evaluate() returns true if any sink node is active.
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // Verify that the graph is now fully active using the strict API.
    assert!(g.all_active().unwrap());
    assert_eq!(g.percent_active().unwrap(), 100.0);
    let total_nodes = g.number_nodes() as f64;
    assert_eq!(g.number_active().unwrap(), total_nodes);
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
    let (g, _data) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();
    let evidence = Evidence::Numerical(0.99);

    // 1. Verify that the graph is fully inactive before evaluation.
    assert!(g.percent_active().is_err());
    assert!(g.number_active().is_err());
    assert!(g.all_active().is_err());

    // 2. Single reasoning: Activate node C (index 3).
    let res = g.evaluate_single_cause(3, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // Now, only node C is active.
    assert_eq!(g.count_known_active(), 1.0);

    // 3. Partial reasoning from C (index 3). This will also activate its descendants F and G.
    let res = g.evaluate_subgraph_from_cause(3, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // Now, nodes C, F, G are active. Total: 3.
    assert_eq!(g.count_known_active(), 3.0);

    // 4. Partial reasoning from B (index 2). This will also activate its descendants E and F.
    // Node F is already active, so only B and E will be newly activated.
    let res = g.evaluate_subgraph_from_cause(2, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // Now, C, F, G (from before) + B, E are active. Total: 5.
    assert_eq!(g.count_known_active(), 5.0);

    // 5. Single reasoning: Deactivate node G (index 7).
    let deactivating_evidence = Evidence::Numerical(0.02);
    let res = g.evaluate_single_cause(7, &deactivating_evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(false));
    // Now, C, F, B, E are active. Total: 4.
    assert_eq!(g.count_known_active(), 4.0);

    // 6. Full reasoning over the entire graph from the root.
    let res = g.evaluate(&evidence);
    assert!(res.is_ok());
    // The graph's evaluate() returns true if any sink node is active.
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // Verify that the graph is now fully active using the strict API.
    assert!(g.all_active().unwrap());
    assert_eq!(g.percent_active().unwrap(), 100.0);
    let total_nodes = g.number_nodes() as f64;
    assert_eq!(g.number_active().unwrap(), total_nodes);
}
