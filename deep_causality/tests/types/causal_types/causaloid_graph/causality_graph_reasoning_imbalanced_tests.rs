/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_graph;
use deep_causality::*;

#[test]
fn test_left_imbalanced_cause_graph() {
    //   root(0) -> A(1), B(2), C(3)
    //   A(1) -> D(4), E(5)
    let (g, _data) = test_utils_graph::get_left_imbalanced_cause_graph();

    // 1. Verify that the graph is fully inactive before evaluation.
    assert!(g.percent_active().is_err());
    assert!(g.number_active().is_err());
    assert!(g.all_active().is_err());

    // 2. Single reasoning: Activate node C (index 3).
    let index_c = 3;
    let evidence = Evidence::Numerical(0.99);
    let res = g.evaluate_single_cause(index_c, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // Now, only node C is active.
    assert_eq!(g.count_known_active(), 1.0);

    // 3. Partial reasoning from A (index 1). This will also activate its descendants D and E.
    let res = g.evaluate_subgraph_from_cause(1, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // Now, nodes C (from before) + A, D, E are active. Total: 4.
    assert_eq!(g.count_known_active(), 4.0);

    // 4. Selective sub-graph reasoning from A(1) to D(4).
    // The path is [1, 4]. These nodes are already active. Re-evaluating them is idempotent.
    let res = g.evaluate_shortest_path_between_causes(1, 4, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // The number of active nodes should not change.
    assert_eq!(g.count_known_active(), 4.0);

    // 5. Single reasoning: Deactivate node A (index 1).
    let index_c = 1;
    let evidence = Evidence::Numerical(0.02);
    let res = g.evaluate_single_cause(index_c, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(false));
    // Now, only C, D, E are active. Total: 3.
    assert_eq!(g.count_known_active(), 3.0);

    // 6. Full reasoning over the entire graph from the root.
    let evidence = Evidence::Numerical(0.99);
    let res = g.evaluate(&evidence);
    dbg!(&res);
    assert!(res.is_ok());
    // The graph's evaluate() returns true if any sink node is active.
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // Verify that the graph is now fully active.
    assert!(g.all_active().unwrap());
    assert_eq!(g.percent_active().unwrap(), 100.0);
    let total_nodes = g.number_nodes() as f64;
    assert_eq!(g.count_known_active(), total_nodes);
}

#[test]
fn test_right_imbalanced_cause_graph() {
    //   root(0) -> A(1), B(2), C(3)
    //   C(3) -> D(4), E(5)
    let (g, _data) = test_utils_graph::get_right_imbalanced_cause_graph();

    // 1. Verify that the graph is fully inactive before evaluation.
    assert!(g.percent_active().is_err());
    assert!(g.number_active().is_err());
    assert!(g.all_active().is_err());

    // 2. Single reasoning: Activate node C (index 3).
    let index_c = 3;
    let evidence = Evidence::Numerical(0.99);
    let res = g.evaluate_single_cause(index_c, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // Now, only node C is active.
    assert_eq!(g.count_known_active(), 1.0);

    // 3. Partial reasoning from C (index 3). This will also activate its descendants D and E.
    let res = g.evaluate_subgraph_from_cause(3, &evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    // Now, nodes C, D, E are active. Total: 3.
    assert_eq!(g.count_known_active(), 3.0);

    // 4. Single reasoning: Deactivate node A (index 1).
    let index_c = 1;
    let evidence = Evidence::Numerical(0.02);
    let res = g.evaluate_single_cause(index_c, &evidence);
    dbg!(&res);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(false));
    // The number of active nodes should not change, as B was already inactive.
    assert_eq!(g.count_known_active(), 3.0);

    // 5. Full reasoning over the entire graph from the root.
    let evidence = Evidence::Numerical(0.99);
    let res = g.evaluate(&evidence);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // Verify that the graph is now fully active.
    assert!(g.all_active().unwrap());
    assert_eq!(g.percent_active().unwrap(), 100.0);
    let total_nodes = g.number_nodes() as f64;
    assert_eq!(g.count_known_active(), total_nodes);
}
