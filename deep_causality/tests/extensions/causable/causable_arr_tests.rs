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

// Helper to activate all causes in a collection for testing purposes.
fn activate_all_causes(col: &[BaseCausaloid]) {
    // A value that ensures the default test causaloid (threshold 0.55) becomes active.
    let evidence = Evidence::Numerical(0.99);
    for cause in col {
        // We call evaluate to set the internal state, but ignore the result for this setup.
        let _ = cause.evaluate(&evidence);
    }
}

#[test]
fn test_get_all_causes_true() {
    let col = get_test_causality_array();
    // Before evaluation, is_active returns an error, so get_all_causes_true will be false.
    assert!(!col.get_all_causes_true().unwrap_or(false));

    activate_all_causes(&col);
    // After activation, the result should be Ok(true).
    assert!(col.get_all_causes_true().unwrap());
}

#[test]
fn test_number_active() {
    let col = get_test_causality_array();
    // Before evaluation, number_active will error.
    assert!(col.number_active().is_err());

    activate_all_causes(&col);
    // After activation, all 10 should be active.
    assert_eq!(10.0, col.number_active().unwrap());
}

#[test]
fn test_percent_active() {
    let col = get_test_causality_array();
    // Before evaluation, percent_active will error.
    assert!(col.percent_active().is_err());

    activate_all_causes(&col);
    assert_eq!(10.0, col.number_active().unwrap());
    assert_eq!(100.0, col.percent_active().unwrap());
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
fn test_get_all_active_and_inactive_causes() {
    let col = get_test_causality_array();

    // 1. Evaluate all causes to be inactive.
    let inactive_evidence = Evidence::Numerical(0.1); // Below threshold of 0.55
    for cause in &col {
        cause.evaluate(&inactive_evidence).unwrap();
    }
    assert_eq!(0, col.get_all_active_causes().unwrap().len());
    assert_eq!(10, col.get_all_inactive_causes().unwrap().len());

    // 2. Evaluate all causes to be active.
    let active_evidence = Evidence::Numerical(0.99); // Above threshold
    for cause in &col {
        cause.evaluate(&active_evidence).unwrap();
    }
    assert_eq!(10, col.get_all_active_causes().unwrap().len());
    assert_eq!(0, col.get_all_inactive_causes().unwrap().len());
}

#[test]
fn test_evaluate_deterministic_propagation() {
    let col = get_test_causality_array();

    // Case 1: All succeed, chain should be deterministically true.
    let evidence_success = Evidence::Numerical(0.99);
    let res_success = col
        .evaluate_deterministic_propagation(&evidence_success)
        .unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let evidence_fail = Evidence::Numerical(0.1);
    let res_fail = col
        .evaluate_deterministic_propagation(&evidence_fail)
        .unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let col = get_test_causality_array();

    // Case 1: All succeed (Deterministic(true) is treated as probability 1.0).
    // The cumulative probability should be 1.0.
    let evidence_success = Evidence::Numerical(0.99);
    let res_success = col
        .evaluate_probabilistic_propagation(&evidence_success)
        .unwrap();
    assert_eq!(res_success, PropagatingEffect::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let evidence_fail = Evidence::Numerical(0.1);
    let res_fail = col
        .evaluate_probabilistic_propagation(&evidence_fail)
        .unwrap();
    assert_eq!(res_fail, PropagatingEffect::Probabilistic(0.0));
}

#[test]
fn test_evaluate_mixed_propagation() {
    let col = get_test_causality_array();

    // Case 1: All succeed, chain remains deterministically true.
    let evidence_success = Evidence::Numerical(0.99);
    let res_success = col.evaluate_mixed_propagation(&evidence_success).unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain becomes deterministically false.
    let evidence_fail = Evidence::Numerical(0.1);
    let res_fail = col.evaluate_mixed_propagation(&evidence_fail).unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_explain() {
    let col = get_test_causality_array();
    activate_all_causes(&col);

    let single_explanation = "\n * Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: Deterministic(true)\n";
    let expected = single_explanation.repeat(10);
    let actual = col.explain();
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
