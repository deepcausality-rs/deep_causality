/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::*;

// Helper to activate all causes in a collection for testing purposes.
fn activate_all_causes(col: &BaseCausaloidVec) {
    // A value that ensures the default test causaloid (threshold 0.55) becomes active.
    let effect = PropagatingEffect::Numerical(0.99);
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
fn test_evaluate_deterministic_propagation() {
    let col = test_utils::get_deterministic_test_causality_vec();

    // Case 1: All succeed, chain should be deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_deterministic(&effect_success, &AggregateLogic::All);
    assert!(res.is_ok());
    let res_success = res.unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain should be deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_deterministic(&effect_fail, &AggregateLogic::All);
    assert!(res.is_ok());
    let res_fail = res.unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_evaluate_probabilistic_propagation() {
    let col = test_utils::get_probabilistic_test_causality_vec();

    // Case 1: All succeed (Deterministic(true) is treated as probability 1.0).
    // The cumulative probability should be 1.0.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_probabilistic(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();
    assert_eq!(res_success, PropagatingEffect::Probabilistic(1.0));

    // Case 2: One fails (Deterministic(false) is treated as probability 0.0).
    // The chain should short-circuit and return a cumulative probability of 0.0.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_probabilistic(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();
    assert_eq!(res_fail, PropagatingEffect::Probabilistic(0.0));
}

#[test]
fn test_evaluate_uncertain_propagation() {
    let col = test_utils::get_uncertain_bool_test_causality_vec();

    // Case 1: All succeed.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_uncertain(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();

    let uncertain_bool = match res_success {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_success),
    };

    // Since all inputs are point(true), the result of AND should be deterministically true.
    let final_bool = uncertain_bool.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(final_bool);

    // Case 2: All fail.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_uncertain(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();

    let uncertain_bool_fail = match res_fail {
        PropagatingEffect::UncertainBool(u) => u,
        _ => panic!("Expected UncertainBool but got {:?}", res_fail),
    };

    // Since all inputs are point(false), the result of AND should be deterministically false.
    let final_bool_fail = uncertain_bool_fail.to_bool(0.5, 0.95, 0.05, 1).unwrap();
    assert!(!final_bool_fail);
}

#[test]
fn test_evaluate_mixed_propagation() {
    let col = test_utils::get_deterministic_test_causality_vec();

    // Case 1: All succeed, chain remains deterministically true.
    let effect_success = PropagatingEffect::Numerical(0.99);
    let res = col.evaluate_mixed(&effect_success, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_success = res.unwrap();
    assert_eq!(res_success, PropagatingEffect::Deterministic(true));

    // Case 2: One fails, chain becomes deterministically false.
    let effect_fail = PropagatingEffect::Numerical(0.1);
    let res = col.evaluate_mixed(&effect_fail, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let res_fail = res.unwrap();
    assert_eq!(res_fail, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_explain() {
    let col = test_utils::get_deterministic_test_causality_vec();
    activate_all_causes(&col);

    let single_explanation = "\n * Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: PropagatingEffect::Deterministic(true)\n";
    let expected = single_explanation.repeat(3);
    let res = col.explain();
    assert!(res.is_ok());
    let actual = res.unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_len() {
    let col = test_utils::get_deterministic_test_causality_vec();
    assert_eq!(3, col.len());
}

#[test]
fn test_is_empty() {
    let col = test_utils::get_deterministic_test_causality_vec();
    assert!(!col.is_empty());
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
    let effect = PropagatingEffect::Numerical(0.0);
    let result = coll.evaluate_deterministic(&effect, &AggregateLogic::All);

    // Assert: This covers the error branch for non-deterministic effects.
    assert!(result.is_err());
    let result = coll.evaluate_deterministic(&effect, &AggregateLogic::All);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains(
            "encountered a non-deterministic effect: PropagatingEffect::Probabilistic(0.0)"
        )
    );
}

#[test]
fn test_evaluate_probabilistic_propagation_success() {
    // Setup: Create two causaloids with specific probabilities to test multiplication.
    fn causal_fn_half(_: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::Probabilistic(0.5))
    }
    let p1 = Causaloid::new(1, causal_fn_half, "p=0.5");

    fn causal_fn_quarter(_: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::Probabilistic(0.25))
    }
    let p2 = Causaloid::new(2, causal_fn_quarter, "p=0.25");

    let coll: Vec<BaseCausaloid> = vec![p1, p2];

    // Act: Evaluate with probabilistic propagation.
    let effect = PropagatingEffect::Numerical(0.0);
    let res = coll.evaluate_probabilistic(&effect, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();

    // Assert: This covers the main logic branch, ensuring probabilities are multiplied.
    assert_eq!(result, PropagatingEffect::Probabilistic(0.125));
}

#[test]
fn test_evaluate_probabilistic_propagation_error_contextual_link() {
    // Setup: A collection containing a ContextualLink causaloid, which is invalid for this function.
    let probabilistic_causaloid = test_utils::get_test_causaloid_probabilistic();
    let contextual_link_causaloid = test_utils::get_test_causaloid_contextual_link();
    let coll: Vec<BaseCausaloid> = vec![probabilistic_causaloid, contextual_link_causaloid];

    // Act: Evaluate with probabilistic propagation.
    let effect = PropagatingEffect::Numerical(0.0);
    let result = coll.evaluate_probabilistic(&effect, &AggregateLogic::All, 0.5);

    // Assert: This covers the error branch for invalid ContextualLink effects.
    assert!(result.is_err());
}
