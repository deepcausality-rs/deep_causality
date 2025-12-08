/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::*;

#[test]
fn test_add() {
    let mut col = test_utils::get_deterministic_test_causality_vec();
    assert_eq!(3, col.len());

    let q = test_utils::get_test_causaloid_deterministic(4);
    col.push(q);
    assert_eq!(4, col.len());
}

#[test]
fn test_get_all_items() {
    let col: BaseCausaloidVec<NumericalValue, bool> =
        test_utils::get_deterministic_test_causality_vec();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_explain() {
    let col = test_utils::get_deterministic_test_causality_vec();
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
    let col: BaseCausaloidVec<NumericalValue, bool> =
        test_utils::get_deterministic_test_causality_vec();
    assert!(col.get_item_by_id(1).is_some());
}

#[test]
fn test_len() {
    let col: BaseCausaloidVec<NumericalValue, bool> =
        test_utils::get_deterministic_test_causality_vec();
    assert_eq!(3, col.len());
}

#[test]
fn test_is_empty() {
    let col: BaseCausaloidVec<NumericalValue, bool> =
        test_utils::get_deterministic_test_causality_vec();
    assert!(!col.is_empty());
}

#[test]
fn test_to_vec() {
    let col: BaseCausaloidVec<NumericalValue, bool> =
        test_utils::get_deterministic_test_causality_vec();
    assert_eq!(3, col.to_vec().len());
}
