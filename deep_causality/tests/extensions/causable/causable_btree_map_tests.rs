/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::BTreeMap;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Type alias for clarity in test functions.
type TestBTreeMap = BTreeMap<i8, BaseCausaloid<NumericalValue, bool>>;

// Helper function to create a standard test BTreeMap.
fn get_test_causality_btree_map_deterministic() -> TestBTreeMap {
    BTreeMap::from([
        (1, get_test_causaloid_deterministic(1)),
        (2, get_test_causaloid_deterministic(2)),
        (3, get_test_causaloid_deterministic(3)),
    ])
}

fn get_test_causality_btree_map_probabilistic() -> BTreeMap<i8, BaseCausaloid<NumericalValue, f64>>
{
    BTreeMap::from([
        (1, get_test_causaloid_probabilistic_bool_output()),
        (2, get_test_causaloid_probabilistic_bool_output()),
        (3, get_test_causaloid_probabilistic_bool_output()),
    ])
}

#[test]
fn test_add() {
    let mut map = get_test_causality_btree_map_deterministic();
    assert_eq!(3, map.len());

    let q = get_test_causaloid_deterministic(4);
    map.insert(4, q);
    assert_eq!(4, map.len());
}

#[test]
fn test_contains() {
    let mut map = get_test_causality_btree_map_deterministic();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));
    assert!(map.get_item_by_id(1).is_some());

    let q = get_test_causaloid_deterministic(4);
    map.insert(4, q);

    assert_eq!(4, map.len());
    assert!(map.contains_key(&4));
}

#[test]
fn test_remove() {
    let mut map = get_test_causality_btree_map_deterministic();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    map.remove(&3);
    assert_eq!(2, map.len());
    assert!(!map.contains_key(&3));
}

#[test]
fn test_get_all_items() {
    let col = get_test_causality_btree_map_deterministic();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let map = get_test_causality_btree_map_deterministic();
    let registry = CausaloidRegistry::new();

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res = map.evaluate_collection(&registry, &effect_success, &AggregateLogic::All, None);
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = map.evaluate_collection(&registry, &effect_fail, &AggregateLogic::All, None);
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Deterministic(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let map = get_test_causality_btree_map_probabilistic();
    let registry = CausaloidRegistry::new();

    // Case 1: All succeed (Deterministic(true) is treated as probability 1.0).
    // The cumulative probability should be 1.0.
    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res = map.evaluate_collection(&registry, &effect_success, &AggregateLogic::All, Some(0.5));
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let effect_fail = PropagatingEffect::from_numerical(0.1);
    let res = map.evaluate_collection(&registry, &effect_fail, &AggregateLogic::All, Some(0.5));
    assert!(!res.is_err());
    assert_eq!(res.value, EffectValue::Probabilistic(0.0));
}

#[test]
fn test_len() {
    let map = get_test_causality_btree_map_deterministic();
    assert_eq!(3, map.len());
}

#[test]
fn test_is_empty() {
    let map = get_test_causality_btree_map_deterministic();
    assert!(!map.is_empty());

    let empty: TestBTreeMap = BTreeMap::new();
    assert!(empty.is_empty())
}

#[test]
fn test_to_vec() {
    let map = get_test_causality_btree_map_deterministic();
    assert_eq!(3, map.to_vec().len());
}
