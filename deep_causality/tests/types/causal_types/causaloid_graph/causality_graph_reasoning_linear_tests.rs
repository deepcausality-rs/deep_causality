/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_graph;
use deep_causality::{Causable, CausableGraph, Evidence, PropagatingEffect};

#[test]
fn test_linear_graph() {
    // Reasons over a linear graph:
    // root(0) -> A(1) -> B(2) -> C(3) ...
    // We assume a linear chain of causality.
    let (g, _data) = test_utils_graph::get_small_linear_graph_and_data();

    // 1. Before evaluation, assert that active status is unknown (returns Err)
    assert!(g.percent_active().is_err());
    assert!(g.number_active().is_err());
    assert!(g.all_active().is_err());

    // 2. Evaluate the graph using the high-level Causable::evaluate method.
    // The synthetic dataset is designed to activate all nodes.
    let evidence = Evidence::Numerical(0.99);
    let res = g.evaluate(&evidence);

    // Assert that evaluation was successful. For a linear graph, the final effect
    // is determined by the state of the last node (the sink).
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // 3. After evaluation, verify that the graph is fully active.
    assert!(g.all_active().unwrap());
    assert_eq!(g.percent_active().unwrap(), 100.0);

    let total_nodes = g.number_nodes() as f64;
    assert_eq!(g.number_active().unwrap(), total_nodes);
}
