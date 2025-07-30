/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::*;

#[test]
fn test_evaluate_subgraph_from_cause() {
    let mut g = CausaloidGraph::new(0);

    // Build a graph: root -> A, root -> B; A -> C
    let root_causaloid = test_utils::get_test_causaloid();
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");

    let causaloid_a = test_utils::get_test_causaloid();
    let idx_a = g
        .add_causaloid(causaloid_a)
        .expect("Failed to add causaloid A");
    g.add_edge(root_index, idx_a).expect("Failed to add edge");

    let causaloid_b = test_utils::get_test_causaloid();
    let idx_b = g
        .add_causaloid(causaloid_b)
        .expect("Failed to add causaloid B");
    g.add_edge(root_index, idx_b).expect("Failed to add edge");

    let causaloid_c = test_utils::get_test_causaloid();
    let idx_c = g
        .add_causaloid(causaloid_c)
        .expect("Failed to add causaloid C");
    g.add_edge(idx_a, idx_c).expect("Failed to add edge");

    g.freeze();

    // 2. Evaluate a subgraph starting from node A. This should activate nodes A and C.
    let effect = PropagatingEffect::Numerical(0.99);
    let res = g.evaluate_subgraph_from_cause(idx_a, &effect);

    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
}

#[test]
fn test_evaluate_subgraph_fails_if_not_frozen() {
    let effect = PropagatingEffect::Numerical(0.99);
    let mut g = CausaloidGraph::new(0);
    let root_causaloid = test_utils::get_test_causaloid();
    let root_index = g.add_root_causaloid(root_causaloid).unwrap();

    // DO NOT call g.freeze()

    let res = g.evaluate_subgraph_from_cause(root_index, &effect);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "CausalityError: Graph is not frozen. Call freeze() first"
    );
}

#[test]
fn test_evaluate_subgraph_fails_if_node_missing() {
    let effect = PropagatingEffect::Numerical(0.99);
    let mut g = CausaloidGraph::new(0); // An empty graph

    // Build a graph: root
    let root_causaloid = test_utils::get_test_causaloid();
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    assert!(g.contains_causaloid(root_index));

    g.freeze(); // Freeze it

    let non_existent_index = 99;
    let res = g.evaluate_subgraph_from_cause(non_existent_index, &effect);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "CausalityError: Graph does not contain start causaloid with index 99"
    );
}
