/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::array;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Helper function to create a standard test array.
// Causaloid doesn't implement Copy, hence the from_fn workaround for array initialization.
pub fn get_test_causality_array(deterministic: bool) -> [BaseCausaloid; 10] {
    if deterministic {
        array::from_fn(|_| get_test_causaloid_deterministic())
    } else {
        array::from_fn(|_| get_test_causaloid_probabilistic())
    }
}

pub fn get_test_causality_array_mixed() -> [BaseCausaloid; 20] {
    let a1 = get_test_causality_array(true);
    let a2 = get_test_causality_array(false);

    // Combine a1 and a2
    a1.into_iter()
        .chain(a2)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

#[test]
fn test_get_all_items() {
    let col = get_test_causality_array(true);
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let col = get_test_causality_array(true);

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res_success = col.evaluate_collection(effect_success);
    assert!(res_success.error.is_none());
    assert_eq!(res_success.value, EffectValue::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res_fail = col.evaluate_collection_monadic(effect_fail);
    assert!(res_fail.error.is_none());
    assert_eq!(res_fail.value, EffectValue::Deterministic(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let col = get_test_causality_array(false);

    let effect_success = PropagatingEffect::from_probabilistic(0.99);
    let res_success = col.evaluate_collection_monadic(effect_success);
    assert!(res_success.error.is_none());
    assert_eq!(res_success.value, EffectValue::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res_fail = col.evaluate_collection_monadic(effect_fail);
    assert!(res_fail.error.is_none());
    assert_eq!(res_fail.value, EffectValue::Probabilistic(0.0));
}

#[test]
fn test_evaluate_mixed_propagation() {
    let col = get_test_causality_array_mixed();

    // Case 1: All succeed, chain remains deterministically true.
    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res_success = col.evaluate_collection_monadic(effect_success);
    assert!(res_success.error.is_none());
    assert_eq!(res_success.value, EffectValue::Deterministic(true));

    // Case 2: One fails, chain becomes deterministically false.
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res_fail = col.evaluate_collection_monadic(effect_fail);
    assert!(res_fail.error.is_none());
    assert_eq!(res_fail.value, EffectValue::Deterministic(false));
}

#[test]
fn test_explain() {
    let col = get_test_causality_array(true);

    let effect = PropagatingEffect::from_numerical(0.99);
    let res = col.evaluate_collection_monadic(effect);
    assert!(res.error.is_none());

    let actual = res.explain();

    let single_explanation = "\n * Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: EffectValue::Deterministic(true)\n";
    let expected = single_explanation.repeat(10);

    assert_eq!(expected, actual);
}

#[test]
fn test_get_item_by_id() {
    let col = get_test_causality_array(true);
    assert!(col.get_item_by_id(1).is_some());
}

#[test]
fn test_len() {
    let col = get_test_causality_array(true);
    assert_eq!(10, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_test_causality_array(true);
    assert!(!col.is_empty());

    let empty: [BaseCausaloid; 0] = [];
    assert!(empty.is_empty());
}

#[test]
fn test_to_vec() {
    let col = get_test_causality_array(true);
    assert_eq!(10, col.to_vec().len());
}
