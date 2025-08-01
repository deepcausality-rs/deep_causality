/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::*;

#[test]
fn test_graph_evaluate() {
    let mut g = CausaloidGraph::new(0);

    // Add root causaloid
    let root_causaloid = test_utils::get_test_causaloid_deterministic();
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    assert!(g.contains_causaloid(root_index));

    // Add causaloid A
    let causaloid_a = test_utils::get_test_causaloid_deterministic();
    let idx_a = g
        .add_causaloid(causaloid_a)
        .expect("Failed to add causaloid A");
    g.add_edge(root_index, idx_a).expect("Failed to add edge");

    // Add causaloid B
    let causaloid_b = test_utils::get_test_causaloid_deterministic();
    let idx_b = g
        .add_causaloid(causaloid_b)
        .expect("Failed to add causaloid B");
    g.add_edge(root_index, idx_b).expect("Failed to add edge");

    // Add causaloid C
    let causaloid_c = test_utils::get_test_causaloid_deterministic();
    let idx_c = g
        .add_causaloid(causaloid_c)
        .expect("Failed to add causaloid C");
    g.add_edge(idx_a, idx_c).expect("Failed to add edge");

    g.freeze();

    // Evaluate the graph using the Causable::evaluate method
    let effect = PropagatingEffect::Numerical(0.99); // A value that will activate all nodes
    let res = g.evaluate(&effect);

    // Assert that evaluation was successful and the effect is as expected.
    // The graph's evaluate returns Deterministic(true) because the sink node C becomes active.
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
}

#[test]
fn test_graph_evaluate_error_conditions() {
    // Test case 1: Graph has no root
    let g: BaseCausalGraph = CausaloidGraph::new(0);
    assert!(g.is_empty());
    assert!(!g.contains_root_causaloid());

    let effect = PropagatingEffect::Numerical(0.99);
    let res = g.evaluate(&effect);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "CausalityError: Cannot evaluate graph: Root node not found."
    );

    // Test case 2: Graph is not frozen
    let mut g = CausaloidGraph::new(0);
    let root_causaloid = test_utils::get_test_causaloid_deterministic();
    g.add_root_causaloid(root_causaloid).unwrap();
    // DO NOT call g.freeze()

    let res = g.evaluate(&effect);
    assert!(res.is_err());
    // The error comes from `evaluate_subgraph_from_cause` which is called by `evaluate`
    assert_eq!(
        res.unwrap_err().to_string(),
        "CausalityError: Graph is not frozen. Call freeze() first"
    );
}
