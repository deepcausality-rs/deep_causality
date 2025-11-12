/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashMap;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Type alias for clarity in test functions.
type TestHashMap = HashMap<i8, BaseCausaloid<NumericalValue, bool>>;

// Helper function to create a standard test HashMap.
fn get_deterministic_test_causality_map() -> TestHashMap {
    HashMap::from([
        (1, get_test_causaloid_deterministic(1)),
        (2, get_test_causaloid_deterministic(2)),
        (3, get_test_causaloid_deterministic(3)),
    ])
}

fn get_probabilistic_test_causality_map() -> HashMap<i8, BaseCausaloid<NumericalValue, f64>> {
    HashMap::from([
        (1, get_test_causaloid_probabilistic_bool_output()),
        (2, get_test_causaloid_probabilistic_bool_output()),
        (3, get_test_causaloid_probabilistic_bool_output()),
    ])
}

#[test]
fn test_add() {
    let mut map = get_deterministic_test_causality_map();
    assert_eq!(3, map.len());

    let q = get_test_causaloid_deterministic(4);
    map.insert(4, q);
    assert_eq!(4, map.len());
}

#[test]
fn test_contains() {
    let mut map = get_deterministic_test_causality_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    let q = get_test_causaloid_deterministic(4);
    map.insert(4, q);
    assert_eq!(4, map.len());
    assert!(map.contains_key(&4));
}

#[test]
fn test_remove() {
    let mut map = get_deterministic_test_causality_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    map.remove(&3);
    assert_eq!(2, map.len());
    assert!(!map.contains_key(&3));
}

#[test]
fn test_get_all_items() {
    let col = get_deterministic_test_causality_map();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let map = get_deterministic_test_causality_map();

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res = map.evaluate_collection(&effect_success, &AggregateLogic::All, None);
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = map.evaluate_collection(&effect_fail, &AggregateLogic::All, None);
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Deterministic(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let map = get_probabilistic_test_causality_map();

    // Case 1: All succeed (Deterministic(true) is treated as probability 1.0).
    // The cumulative probability should be 1.0.
    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res = map.evaluate_collection(&effect_success, &AggregateLogic::All, Some(0.5));
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = map.evaluate_collection(&effect_fail, &AggregateLogic::All, Some(0.5));
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Probabilistic(0.0));
}

#[test]
fn test_explain() {
    let map = get_deterministic_test_causality_map();
    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res = map.evaluate_collection(&effect_success, &AggregateLogic::All, None);

    assert!(!res.is_err());
    let actual_explanation = res.explain();
    dbg!(&actual_explanation);

    let expected_final_value = format!("Final Value: {:?}\n", res.value);
    assert!(actual_explanation.contains(&expected_final_value));
    assert!(actual_explanation.contains("--- Logs ---\n"));

    // For each causaloid (id 1, 2, 3)
    for i in 1..=3 {
        let incoming_log = format!("Causaloid {}: Incoming effect: Numerical(0.99)", i);
        let output_log = format!("Causaloid {}: Output effect: Deterministic(true)", i);
        assert!(actual_explanation.contains(&incoming_log));
        assert!(actual_explanation.contains(&output_log));
    }
    // Also the collection's own log
    // assert!(actual_explanation.contains(&format!("Causaloid {}: Incoming effect for Collection: Numerical(0.99)", res.id)));
}

#[test]
fn test_get_item_by_id() {
    let map = get_deterministic_test_causality_map();
    assert!(map.contains_key(&1));
    assert!(map.contains_key(&2));
    assert!(map.contains_key(&3));

    assert!(map.get_item_by_id(1).is_some());
}

#[test]
fn test_len() {
    let map = get_deterministic_test_causality_map();
    assert_eq!(3, map.len());
}

#[test]
fn test_is_empty() {
    let map = get_deterministic_test_causality_map();
    assert!(!map.is_empty());
}

#[test]
fn test_to_vec() {
    let map = get_deterministic_test_causality_map();
    assert_eq!(3, map.to_vec().len());
}
