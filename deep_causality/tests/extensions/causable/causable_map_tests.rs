/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashMap;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Type alias for clarity in test functions.
type TestHashMap = HashMap<i8, BaseCausaloid>;

// Helper function to create a standard test HashMap.
fn get_test_causality_map() -> TestHashMap {
    HashMap::from([
        (1, get_test_causaloid()),
        (2, get_test_causaloid()),
        (3, get_test_causaloid()),
    ])
}

fn get_mixed_test_causality_map() -> TestHashMap {
    HashMap::from([
        (1, get_test_causaloid_deterministic_true()),
        (2, get_test_causaloid_probabilistic()),
        (3, get_test_causaloid_deterministic_true()),
    ])
}

fn get_mixed_test_halting_causality_map() -> TestHashMap {
    HashMap::from([
        (1, get_test_causaloid_deterministic_true()),
        (2, get_test_causaloid_deterministic_true()),
        (3, get_test_causaloid_halting()),
    ])
}

fn get_mixed_test_ctx_link_causality_map() -> TestHashMap {
    HashMap::from([
        (1, get_test_causaloid_deterministic_false()),
        (2, get_test_causaloid_deterministic_true()),
        (3, get_test_causaloid_contextual_link()),
    ])
}

// Helper to activate all causes in a collection for testing purposes.
fn activate_all_causes(map: &TestHashMap) {
    // A value that ensures the default test causaloid (threshold 0.55) becomes active.
    let evidence = Evidence::Numerical(0.99);
    for cause in map.values() {
        // We call evaluate to set the internal state, but ignore the result for this setup.
        let _ = cause.evaluate(&evidence);
    }
}

#[test]
fn test_add() {
    let mut map = get_test_causality_map();
    assert_eq!(3, map.len());

    let q = get_test_causaloid();
    map.insert(4, q);
    assert_eq!(4, map.len());
}

#[test]
fn test_contains() {
    let mut map = get_test_causality_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    let q = get_test_causaloid();
    map.insert(4, q);
    assert_eq!(4, map.len());
    assert!(map.contains_key(&4));
}

#[test]
fn test_remove() {
    let mut map = get_test_causality_map();
    assert_eq!(3, map.len());
    assert!(map.contains_key(&1));

    map.remove(&3);
    assert_eq!(2, map.len());
    assert!(!map.contains_key(&3));
}

#[test]
fn test_get_all_causes_true() {
    let map = get_test_causality_map();
    // Before evaluation, is_active returns an error, so get_all_causes_true will be false.
    assert!(!map.get_all_causes_true().unwrap_or(false));

    activate_all_causes(&map);
    // After activation, the result should be Ok(true).
    assert!(map.get_all_causes_true().unwrap());
}

#[test]
fn test_number_active() {
    let map = get_test_causality_map();
    // Before evaluation, number_active will error.
    assert!(map.number_active().is_err());

    activate_all_causes(&map);
    // After activation, all 3 should be active.
    assert_eq!(3.0, map.number_active().unwrap());
}

#[test]
fn test_percent_active() {
    let map = get_test_causality_map();
    // Before evaluation, percent_active will error.
    assert!(map.percent_active().is_err());

    activate_all_causes(&map);
    assert_eq!(3.0, map.number_active().unwrap());
    assert_eq!(100.0, map.percent_active().unwrap());
}

#[test]
fn test_get_all_items() {
    let col = get_test_causality_map();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_get_all_active_and_inactive_causes() {
    let map = get_test_causality_map();

    // 1. Evaluate all causes to be inactive.
    let inactive_evidence = Evidence::Numerical(0.1); // Below threshold of 0.55
    for cause in map.values() {
        cause.evaluate(&inactive_evidence).unwrap();
    }
    assert_eq!(0, map.get_all_active_causes().unwrap().len());
    assert_eq!(3, map.get_all_inactive_causes().unwrap().len());

    // 2. Evaluate all causes to be active.
    let active_evidence = Evidence::Numerical(0.99); // Above threshold
    for cause in map.values() {
        cause.evaluate(&active_evidence).unwrap();
    }
    assert_eq!(3, map.get_all_active_causes().unwrap().len());
    assert_eq!(0, map.get_all_inactive_causes().unwrap().len());
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let map = get_test_causality_map();

    // Case 1: All succeed, chain should be deterministically true.
    let evidence_success = Evidence::Numerical(0.99);
    let res_success = map
        .evaluate_deterministic_propagation(&evidence_success)
        .unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let evidence_fail = Evidence::Numerical(0.1);
    let res_fail = map
        .evaluate_deterministic_propagation(&evidence_fail)
        .unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let map = get_test_causality_map();

    // Case 1: All succeed (Deterministic(true) is treated as probability 1.0).
    // The cumulative probability should be 1.0.
    let evidence_success = Evidence::Numerical(0.99);
    let res_success = map
        .evaluate_probabilistic_propagation(&evidence_success)
        .unwrap();
    assert_eq!(res_success, PropagatingEffect::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let evidence_fail = Evidence::Numerical(0.1);
    let res_fail = map
        .evaluate_probabilistic_propagation(&evidence_fail)
        .unwrap();
    assert_eq!(res_fail, PropagatingEffect::Probabilistic(0.0));
}

#[test]
fn test_evaluate_mixed_propagation() {
    let map = get_mixed_test_causality_map();

    // Case 1: All succeed, chain remains deterministically true.
    let evidence_success = Evidence::Numerical(0.99);
    let res_success = map.evaluate_mixed_propagation(&evidence_success).unwrap();
    assert_eq!(res_success, PropagatingEffect::Probabilistic(0.0));
}

#[test]
fn test_evaluate_mixed_propagation_halted() {
    let map = get_mixed_test_halting_causality_map();

    // Case 2: One fails, chain becomes deterministically false.
    let evidence_fail = Evidence::Numerical(0.1);
    let res = map.evaluate_mixed_propagation(&evidence_fail).unwrap();
    assert_eq!(res, PropagatingEffect::Halting);
}

#[test]
fn test_evaluate_mixed_propagation_err() {
    let map = get_mixed_test_ctx_link_causality_map();

    // Case 2: One fails, chain becomes deterministically false.
    let evidence_fail = Evidence::Numerical(0.1);
    let res_fail = map.evaluate_mixed_propagation(&evidence_fail);
    assert!(res_fail.is_err());
}

#[test]
fn test_explain() {
    let map = get_test_causality_map();
    activate_all_causes(&map);

    let single_explanation = "Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: Deterministic(true)";
    let actual = map.explain().unwrap();

    // HashMap iteration order is not guaranteed.
    // We check that the explanation for each of the 3 causes is present.
    assert_eq!(actual.matches(single_explanation).count(), 3);
}

#[test]
fn test_len() {
    let map = get_test_causality_map();
    assert_eq!(3, map.len());
}

#[test]
fn test_is_empty() {
    let map = get_test_causality_map();
    assert!(!map.is_empty());
}

#[test]
fn test_to_vec() {
    let map = get_test_causality_map();
    assert_eq!(3, map.to_vec().len());
}
