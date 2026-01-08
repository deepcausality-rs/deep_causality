/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::VecDeque;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Helper function to create a standard test VecDeque.
fn get_deterministic_test_causality_vec_deque() -> VecDeque<BaseCausaloid<NumericalValue, bool>> {
    VecDeque::from_iter([
        get_test_causaloid_deterministic(1),
        get_test_causaloid_deterministic(2),
        get_test_causaloid_deterministic(3),
    ])
}

fn get_probabilistic_test_causality_vec_deque() -> VecDeque<BaseCausaloid<NumericalValue, f64>> {
    VecDeque::from_iter([
        get_test_causaloid_probabilistic_bool_output(),
        get_test_causaloid_probabilistic_bool_output(),
        get_test_causaloid_probabilistic_bool_output(),
    ])
}

#[test]
fn test_add() {
    let mut col = get_deterministic_test_causality_vec_deque();
    assert_eq!(3, col.len());

    let q = get_test_causaloid_deterministic(4);
    col.push_back(q);
    assert_eq!(4, col.len());
}

#[test]
fn test_get_all_items() {
    let col: VecDeque<BaseCausaloid<NumericalValue, bool>> =
        get_deterministic_test_causality_vec_deque();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let col = get_deterministic_test_causality_vec_deque();

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::from_value(0.99);
    let res = col.evaluate_collection(&effect_success, &AggregateLogic::All, None);
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Value(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::from_value(0.1);
    let res = col.evaluate_collection(&effect_fail, &AggregateLogic::All, None);
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Value(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let col = get_probabilistic_test_causality_vec_deque();

    // Case 1: All succeed (Boolean(true) is treated as probability 1.0).
    // The cumulative probability should be 1.0.
    let effect_success = PropagatingEffect::from_value(0.99);
    let res = col.evaluate_collection(&effect_success, &AggregateLogic::All, Some(0.5));
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Value(1.0));

    // Case 2: One fails (Boolean(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let effect_fail = PropagatingEffect::from_value(0.1);
    let res = col.evaluate_collection(&effect_fail, &AggregateLogic::All, Some(0.5));
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Value(0.0));
}

#[test]
fn test_explain() {
    let col = get_deterministic_test_causality_vec_deque();
    let effect_success = PropagatingEffect::from_value(0.99);
    let res = col.evaluate_collection(&effect_success, &AggregateLogic::All, None);

    assert!(!res.is_err());
    let actual_explanation = res.explain();
    dbg!(&actual_explanation);

    let expected_final_value = format!("Final Value: {:?}\n", res.value);
    assert!(actual_explanation.contains(&expected_final_value));
    assert!(actual_explanation.contains("--- Logs ---\n"));

    // For each causaloid (id 1, 2, 3)
    for i in 1..=3 {
        let incoming_log = format!("Causaloid {}: Incoming effect: Value(0.99)", i);
        let output_log = format!("Causaloid {}: Outgoing effect: Value(true)", i);
        assert!(actual_explanation.contains(&incoming_log));
        assert!(actual_explanation.contains(&output_log));
    }
}

#[test]
fn test_get_item_by_id() {
    let col: VecDeque<BaseCausaloid<NumericalValue, bool>> =
        get_deterministic_test_causality_vec_deque();
    assert!(col.get_item_by_id(1).is_some());
}

#[test]
fn test_len() {
    let col: VecDeque<BaseCausaloid<NumericalValue, bool>> =
        get_deterministic_test_causality_vec_deque();
    assert_eq!(3, col.len());
}

#[test]
fn test_is_empty() {
    let col: VecDeque<BaseCausaloid<NumericalValue, bool>> =
        get_deterministic_test_causality_vec_deque();
    assert!(!col.is_empty());
}

#[test]
fn test_to_vec() {
    let col: VecDeque<BaseCausaloid<NumericalValue, bool>> =
        get_deterministic_test_causality_vec_deque();
    assert_eq!(3, col.to_vec().len());
}
