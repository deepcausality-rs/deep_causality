/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::*;

#[test]
fn test_evaluate_single_cause_success() {
    let mut g = CausaloidGraph::new(0);
    let causaloid = test_utils::get_test_causaloid();
    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    g.freeze(); // Reasoning requires a frozen graph

    // Before evaluation, the graph's active state is unknown.
    assert!(g.percent_active().is_err());
    assert!(g.all_active().is_err());

    // Evaluate the node using the high-level graph API.
    let evidence = Evidence::Numerical(0.99);
    let res = g.evaluate_single_cause(index, &evidence);

    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));

    // After evaluation, the graph's state can be determined.
    assert_eq!(g.percent_active().unwrap(), 100.0);
    assert!(g.all_active().unwrap());
    assert_eq!(g.number_active().unwrap(), 1.0);
}

#[test]
fn test_evaluate_single_cause_error_conditions() {
    let evidence = Evidence::Numerical(0.99);

    // Case 1: Node does not exist in the graph.
    let g: BaseCausalGraph = CausaloidGraph::new(0);
    let non_existent_index = 99;
    let res = g.evaluate_single_cause(non_existent_index, &evidence);

    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "CausalityError: Causaloid with index 99 not found in graph"
    );

    // Case 2: The causaloid itself returns an error during evaluation.
    let mut g = CausaloidGraph::new(0);
    let error_causaloid = test_utils::get_test_error_causaloid();
    let index = g
        .add_causaloid(error_causaloid)
        .expect("Failed to add causaloid");
    g.freeze();

    let res = g.evaluate_single_cause(index, &evidence);

    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "CausalityError: Test error");
}
