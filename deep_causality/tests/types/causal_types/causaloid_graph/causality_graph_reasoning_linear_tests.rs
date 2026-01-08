/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils_graph;
use deep_causality::{MonadicCausableGraphReasoning, PropagatingEffect};

#[test]
fn test_linear_graph() {
    // Reasons over a linear graph:
    // root(0) -> A(1) -> B(2) -> C(3) ...
    // We assume a linear chain of causality.
    let g = test_utils_graph::build_linear_graph(5);

    // 2. Evaluate the graph using the high-level evaluation method directly on the graph.
    // The synthetic dataset is designed to activate all nodes.
    let effect = PropagatingEffect::from_value(0.99);
    // Start from root (index 0)
    let res = g.evaluate_subgraph_from_cause(0, &effect);
    dbg!(&res);
    // Assert that evaluation was successful.
    assert!(res.is_ok());
}
