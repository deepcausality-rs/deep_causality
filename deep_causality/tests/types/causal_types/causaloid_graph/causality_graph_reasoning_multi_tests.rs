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
    let effect = PropagatingEffect::Numerical(0.99);

    // 2. Single reasoning: Activate node B (index 2).
    let res = g.evaluate_single_cause(2, &effect);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // 3. Partial reasoning from B (index 2). This will also activate its descendant C (index 3).
    let res = g.evaluate_subgraph_from_cause(2, &effect);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // 4. Single reasoning: Deactivate node C (index 3).
    let deactivating_effect = PropagatingEffect::Numerical(0.02);
    let res = g.evaluate_single_cause(3, &deactivating_effect);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(false));

    // 5. Full reasoning over the entire graph from the root.
    let res = g.evaluate(&effect);
    assert!(res.is_ok());
    // The graph's evaluate() returns true if any sink node is active.
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
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
    let effect = PropagatingEffect::Numerical(0.99);

    // 2. Single reasoning: Activate node C (index 3).
    let res = g.evaluate_single_cause(3, &effect);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // 3. Partial reasoning from C (index 3). This will also activate its descendants F and G.
    let res = g.evaluate_subgraph_from_cause(3, &effect);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // 4. Partial reasoning from B (index 2). This will also activate its descendants E and F.
    // Node F is already active, so only B and E will be newly activated.
    let res = g.evaluate_subgraph_from_cause(2, &effect);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // 5. Single reasoning: Deactivate node G (index 7).
    let deactivating_effect = PropagatingEffect::Numerical(0.02);
    let res = g.evaluate_single_cause(7, &deactivating_effect);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(false));

    // 6. Full reasoning over the entire graph from the root.
    let res = g.evaluate(&effect);
    assert!(res.is_ok());
    // The graph's evaluate() returns true if any sink node is active.
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
}
