/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::*;

// Helper to activate all causes in a collection for testing purposes.
fn activate_all_causes(col: &BaseCausaloidVec) {
    // A value that ensures the default test causaloid (threshold 0.55) becomes active.
    let effect = PropagatingEffect::from_numerical(0.99);
    for cause in col {
        // We call evaluate to set the internal state, but ignore the result for this setup.
        let _ = cause.evaluate(&effect);
    }
}

#[test]
fn test_add() {
    let mut col = test_utils::get_deterministic_test_causality_vec();
    assert_eq!(3, col.len());

    let q = test_utils::get_test_causaloid_deterministic();
    col.push(q);
    assert_eq!(4, col.len());
}

#[test]
fn test_get_all_items() {
    let col = test_utils::get_deterministic_test_causality_vec();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_explain() {
    let col = test_utils::get_deterministic_test_causality_vec();
    activate_all_causes(&col);

    let single_explanation = "\n * Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: PropagatingEffect::from_deterministic(true)\n";
    let expected = single_explanation.repeat(3);
    let res = col.explain();
    assert!(res.is_ok());
    let actual = res.unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_get_item_by_id() {
    let col = test_utils::get_deterministic_test_causality_vec();
    assert!(col.get_item_by_id(1).is_some());
}

#[test]
fn test_len() {
    let col = test_utils::get_deterministic_test_causality_vec();
    assert_eq!(3, col.len());
    assert_eq!(CausableCollectionReasoning::len(&col), 3);
}

#[test]
fn test_is_empty() {
    let col = test_utils::get_deterministic_test_causality_vec();
    assert!(!col.is_empty());
    assert!(!CausableCollectionReasoning::is_empty(&col));
}

#[test]
fn test_to_vec() {
    let col = test_utils::get_deterministic_test_causality_vec();
    assert_eq!(3, col.to_vec().len());
}

// --- Tests for CausableReasoning Trait ---

#[test]
fn test_evaluate_deterministic_propagation_error_non_deterministic_effect() {
    // Setup: A collection containing a non-deterministic (probabilistic) causaloid.
    let true_causaloid = test_utils::get_test_causaloid_deterministic_true();
    let probabilistic_causaloid = test_utils::get_test_causaloid_probabilistic();
    let coll: Vec<BaseCausaloid> = vec![true_causaloid, probabilistic_causaloid];

    // Act: Evaluate with deterministic propagation, which expects only deterministic effects.
    let effect = PropagatingEffect::from_numerical(0.0);
    let result = coll.evaluate_deterministic(&effect, &AggregateLogic::All);

    // Assert: This covers the error branch for non-deterministic effects.
    assert!(result.is_err());
    let result = coll.evaluate_deterministic(&effect, &AggregateLogic::All);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains(
            "encountered a non-deterministic effect: PropagatingEffect::from_probabilistic(0.0)"
        )
    );
}

#[test]
fn test_evaluate_probabilistic_propagation_success() {
    // Setup: Create two causaloids with specific probabilities to test multiplication.
    fn causal_fn_half(_: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::from_probabilistic(0.5))
    }
    let p1 = Causaloid::new(1, causal_fn_half, "p=0.5");

    fn causal_fn_quarter(_: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::from_probabilistic(0.25))
    }
    let p2 = Causaloid::new(2, causal_fn_quarter, "p=0.25");

    let coll: Vec<BaseCausaloid> = vec![p1, p2];

    // Act: Evaluate with probabilistic propagation.
    let effect = PropagatingEffect::from_numerical(0.0);
    let res = coll.evaluate_probabilistic(&effect, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();

    // Assert: This covers the main logic branch, ensuring probabilities are multiplied.
    assert_eq!(result, PropagatingEffect::from_probabilistic(0.125));
}

#[test]
fn test_evaluate_probabilistic_propagation_error_contextual_link() {
    // Setup: A collection containing a ContextualLink causaloid, which is invalid for this function.
    let probabilistic_causaloid = test_utils::get_test_causaloid_probabilistic();
    let contextual_link_causaloid = test_utils::get_test_causaloid_contextual_link();
    let coll: Vec<BaseCausaloid> = vec![probabilistic_causaloid, contextual_link_causaloid];

    // Act: Evaluate with probabilistic propagation.
    let effect = PropagatingEffect::from_numerical(0.0);
    let result = coll.evaluate_probabilistic(&effect, &AggregateLogic::All, 0.5);

    // Assert: This covers the error branch for invalid ContextualLink effects.
    assert!(result.is_err());
}
