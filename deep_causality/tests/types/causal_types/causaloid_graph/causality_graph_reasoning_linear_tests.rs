/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_graph;
use deep_causality::{Causable, Evidence, PropagatingEffect};

#[test]
fn test_linear_graph() {
    // Reasons over a linear graph:
    // root(0) -> A(1) -> B(2) -> C(3) ...
    // We assume a linear chain of causality.
    let (g, _data) = test_utils_graph::get_small_linear_graph_and_data();

    // 2. Evaluate the graph using the high-level Causable::evaluate method.
    // The synthetic dataset is designed to activate all nodes.
    let evidence = Evidence::Numerical(0.99);
    let res = g.evaluate(&evidence);

    // Assert that evaluation was successful.
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
}
