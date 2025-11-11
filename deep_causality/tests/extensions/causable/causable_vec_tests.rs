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
    let registry = CausaloidRegistry::new();
    let effect_success = PropagatingEffect::from_numerical(0.99);
    let res = col.evaluate_collection(&registry, &effect_success, &AggregateLogic::All, None);

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

// --- Tests for CausableReasoning Trait ---

#[test]
fn test_evaluate_probabilistic_propagation_success() {
    // Setup: Create two causaloids with specific probabilities to test multiplication.
    fn causal_fn_half(_: NumericalValue) -> Result<CausalFnOutput<f64>, CausalityError> {
        Ok(CausalFnOutput {
            output: 0.5,
            log: CausalEffectLog::new(),
        })
    }
    let p1 = Causaloid::new(1, causal_fn_half, "p=0.5");

    fn causal_fn_quarter(_: NumericalValue) -> Result<CausalFnOutput<f64>, CausalityError> {
        Ok(CausalFnOutput {
            output: 0.25,
            log: CausalEffectLog::new(),
        })
    }
    let p2 = Causaloid::new(2, causal_fn_quarter, "p=0.25");

    let coll: Vec<BaseCausaloid<NumericalValue, f64>> = vec![p1, p2];
    let registry = CausaloidRegistry::new();

    // Act: Evaluate with probabilistic propagation.
    let effect = PropagatingEffect::from_numerical(0.0);
    let res = coll.evaluate_collection(&registry, &effect, &AggregateLogic::All, Some(0.5));
    assert!(!res.is_err());
    let result = res.value;

    // Assert: This covers the main logic branch, ensuring probabilities are multiplied.
    assert_eq!(result, EffectValue::Probabilistic(0.125));
}
