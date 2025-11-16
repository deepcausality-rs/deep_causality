/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::*;

#[test]
fn test_evaluate_single_cause_success() {
    let mut g = CausaloidGraph::new(0);
    let causaloid = test_utils::get_test_causaloid_deterministic(0);
    let index = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    g.freeze(); // Reasoning requires a frozen graph

    // Evaluate the node using the high-level graph API.
    let effect = PropagatingEffect::from_numerical(0.99);
    let res = g.evaluate_single_cause(index, &effect);
    dbg!(&res);
    assert!(res.is_ok());
    assert_eq!(res.value, EffectValue::Deterministic(true));
}

#[test]
fn test_evaluate_single_causaloid_not_found_error() {
    // Case 1: Node does not exist in the graph.
    let mut g: BaseCausalGraph = CausaloidGraph::new(0);
    let non_existent_index = 99;
    g.freeze(); // Reasoning requires a frozen graph

    let effect = PropagatingEffect::from_numerical(0.99);
    let res = g.evaluate_single_cause(non_existent_index, &effect);
    dbg!(&res);

    assert!(res.is_err());
    assert!(
        res.error
            .unwrap()
            .to_string()
            .contains("CausalityError: Causaloid with index 99 not found in graph"),
    );
}

#[test]
fn test_evaluate_single_eval_error() {
    // Case 2: The causaloid itself returns an error during evaluation.
    let mut g = CausaloidGraph::new(0);
    let error_causaloid = test_utils::get_test_error_causaloid();
    let index = g
        .add_causaloid(error_causaloid)
        .expect("Failed to add causaloid");
    g.freeze(); // Reasoning requires a frozen graph

    let effect = PropagatingEffect::from_deterministic(false);
    let res = g.evaluate_single_cause(index, &effect);
    dbg!(&res);

    assert!(res.is_err());
    assert_eq!(res.error.unwrap().to_string(), "CausalityError: Test error");
}
