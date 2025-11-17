/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::array;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Helper function to create a standard test array.
// Causaloid doesn't implement Copy, hence the from_fn workaround for array initialization.
pub fn get_test_causality_array_bool_out() -> [BaseCausaloid<NumericalValue, bool>; 10] {
    array::from_fn(|i| get_test_causaloid_deterministic(i as u64))
}

pub fn get_test_causality_array_numerical_value_out()
-> [BaseCausaloid<NumericalValue, NumericalValue>; 10] {
    array::from_fn(|_| get_test_causaloid_probabilistic_bool_output())
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let col = get_test_causality_array_bool_out();

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res = col.evaluate_collection(&effect_success, &AggregateLogic::All, None);
    dbg!(&res);
    assert!(!res.is_err()); // Check for no error
    assert_eq!(res.value, EffectValue::Boolean(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_collection(&effect_fail, &AggregateLogic::All, Some(1.0));
    assert!(!res.is_err()); // Check for no error
    assert_eq!(res.value, EffectValue::Boolean(false));

    // Case 3: An incorrect input effect would trigger an error.
    let effect_fail = PropagatingEffect::from_contextual_link(1, 1); // Fixed argument count
    let res = col.evaluate_collection(&effect_fail, &AggregateLogic::All, Some(1.0));
    assert!(res.is_err());
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let col = get_test_causality_array_numerical_value_out();

    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res = col.evaluate_collection(&effect_success, &AggregateLogic::All, Some(0.5));
    dbg!(&res);
    assert!(!res.is_err()); // Check for no error
    assert_eq!(res.value, EffectValue::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_collection(&effect_fail, &AggregateLogic::All, Some(0.5));
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Probabilistic(0.0));
}

#[test]
fn test_get_all_items() {
    let col = get_test_causality_array_bool_out();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_get_item_by_id() {
    let col = get_test_causality_array_bool_out();
    assert!(col.get_item_by_id(1).is_some());
}

#[test]
fn test_len() {
    let col = get_test_causality_array_bool_out();
    assert_eq!(10, col.len());
}

#[test]
fn test_to_vec() {
    let col = get_test_causality_array_bool_out();
    assert_eq!(10, col.to_vec().len());
}
