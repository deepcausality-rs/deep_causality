/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::array;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Helper function to create a standard test array.
// Causaloid doesn't implement Copy, hence the from_fn workaround for array initialization.
pub fn get_test_causality_array() -> [BaseCausaloid; 10] {
    array::from_fn(|_| get_test_causaloid())
}

#[test]
fn test_get_all_items() {
    let col = get_test_causality_array();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let col = get_test_causality_array();

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res_success = col
        .evaluate_deterministic_propagation(&effect_success, &AggregateLogic::All)
        .unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res_fail = col
        .evaluate_deterministic_propagation(&effect_fail, &AggregateLogic::All)
        .unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let col = get_test_causality_array();

    // Case 1: All succeed (Deterministic(true) is treated as probability 1.0).
    // The cumulative probability should be 1.0.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res_success = col
        .evaluate_probabilistic_propagation(&effect_success, &AggregateLogic::All)
        .unwrap();
    assert_eq!(res_success, PropagatingEffect::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res_fail = col
        .evaluate_probabilistic_propagation(&effect_fail, &AggregateLogic::All)
        .unwrap();
    assert_eq!(res_fail, PropagatingEffect::Probabilistic(0.0));
}

#[test]
fn test_evaluate_mixed_propagation() {
    let col = get_test_causality_array();

    // Case 1: All succeed, chain remains deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res_success = col
        .evaluate_mixed_propagation(&effect_success, &AggregateLogic::All)
        .unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain becomes deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res_fail = col
        .evaluate_mixed_propagation(&effect_fail, &AggregateLogic::All)
        .unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_explain() {
    let col = get_test_causality_array();

    let effect = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_deterministic_propagation(&effect, &AggregateLogic::All);
    assert!(res.is_ok());

    let res = col.explain();
    dbg!(&res);
    assert!(res.is_ok());
    let actual = col.explain().unwrap();

    let single_explanation = "\n * Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: PropagatingEffect::Deterministic(true)\n";
    let expected = single_explanation.repeat(10);

    assert_eq!(expected, actual);
}

#[test]
fn test_len() {
    let col = get_test_causality_array();
    assert_eq!(10, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_test_causality_array();
    assert!(!col.is_empty());
}

#[test]
fn test_to_vec() {
    let col = get_test_causality_array();
    assert_eq!(10, col.to_vec().len());
}
