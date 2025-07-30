/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::BTreeMap;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Type alias for clarity in test functions.
type TestBTreeMap = BTreeMap<i8, BaseCausaloid>;

// Helper function to create a standard test BTreeMap.
fn get_test_causality_btree_map() -> TestBTreeMap {
    BTreeMap::from([
        (1, get_test_causaloid()),
        (2, get_test_causaloid()),
        (3, get_test_causaloid()),
    ])
}

// Helper to activate all causes in a collection for testing purposes.
fn activate_all_causes(map: &TestBTreeMap) {
    // A value that ensures the default test causaloid (threshold 0.55) becomes active.
    let effect = PropagatingEffect::Numerical(0.99);
    for cause in map.values() {
        // We call evaluate to set the internal state, but ignore the result for this setup.
        let _ = cause.evaluate(&effect);
    }
}

#[test]
fn test_add() {
    let mut map = get_test_causality_btree_map();
    assert_eq!(3, map.len());

    let q = get_test_causaloid();
    map.insert(4, q);
    assert_eq!(4, map.len());
}

#[test]
fn test_contains() {
    let mut map = get_test_causality_btree_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    let q = get_test_causaloid();
    map.insert(4, q);
    assert_eq!(4, map.len());
    assert!(map.contains_key(&4));
}

#[test]
fn test_remove() {
    let mut map = get_test_causality_btree_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    map.remove(&3);
    assert_eq!(2, map.len());
    assert!(!map.contains_key(&3));
}

#[test]
fn test_get_all_items() {
    let col = get_test_causality_btree_map();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let map = get_test_causality_btree_map();

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res_success = map
        .evaluate_deterministic_propagation(&effect_success)
        .unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res_fail = map
        .evaluate_deterministic_propagation(&effect_fail)
        .unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let map = get_test_causality_btree_map();

    // Case 1: All succeed (Deterministic(true) is treated as probability 1.0).
    // The cumulative probability should be 1.0.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res_success = map
        .evaluate_probabilistic_propagation(&effect_success)
        .unwrap();
    assert_eq!(res_success, PropagatingEffect::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res_fail = map
        .evaluate_probabilistic_propagation(&effect_fail)
        .unwrap();
    assert_eq!(res_fail, PropagatingEffect::Probabilistic(0.0));
}

#[test]
fn test_evaluate_mixed_propagation() {
    let map = get_test_causality_btree_map();

    // Case 1: All succeed, chain remains deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res_success = map.evaluate_mixed_propagation(&effect_success).unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain becomes deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res_fail = map.evaluate_mixed_propagation(&effect_fail).unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_explain() {
    let map = get_test_causality_btree_map();
    activate_all_causes(&map);

    let single_explanation = "Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: PropagatingEffect::Deterministic(true)";
    // BTreeMap iterates in key-sorted order, so the output is predictable.
    let expected = format!(
        "\n * {}\n\n * {}\n\n * {}\n",
        single_explanation, single_explanation, single_explanation
    );
    let actual = map.explain().unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_len() {
    let map = get_test_causality_btree_map();
    assert_eq!(3, map.len());
}

#[test]
fn test_is_empty() {
    let map = get_test_causality_btree_map();
    assert!(!map.is_empty());
}

#[test]
fn test_to_vec() {
    let map = get_test_causality_btree_map();
    assert_eq!(3, map.to_vec().len());
}
