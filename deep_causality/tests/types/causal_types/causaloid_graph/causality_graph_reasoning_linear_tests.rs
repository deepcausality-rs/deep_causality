/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils_graph;
use deep_causality::{Causaloid, MonadicCausable, PropagatingEffect};
use std::sync::Arc;

#[test]
fn test_linear_graph() {
    // Reasons over a linear graph:
    // root(0) -> A(1) -> B(2) -> C(3) ...
    // We assume a linear chain of causality.
    let causaloid = Causaloid::from_causal_graph(
        0,
        "Test Causality Graph",
        Arc::from(test_utils_graph::build_linear_graph(5)),
    );

    // 2. Evaluate the graph using the high-level Causable::evaluate method.
    // The synthetic dataset is designed to activate all nodes.
    let effect = PropagatingEffect::from_numerical(0.99);
    let res = causaloid.evaluate(&effect);
    dbg!(&res);
    // Assert that evaluation was successful.
    assert!(res.is_ok());
}
