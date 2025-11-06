/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::array;

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

// Helper function to create a standard test array of deterministic causaloids.
pub fn get_test_causality_array_deterministic(all_true: bool) -> [BaseCausaloid; 10] {
    if all_true {
        array::from_fn(|_| get_test_causaloid_deterministic_true())
    } else {
        array::from_fn(|_| get_test_causaloid_deterministic_false())
    }
}

// Helper function to create a standard test array of probabilistic causaloids.
// The causaloids will return 1.0 (true) or 0.0 (false) based on the input effect.
pub fn get_test_causality_array_probabilistic() -> [BaseCausaloid; 10] {
    array::from_fn(|_| get_test_causaloid_probabilistic())
}

// Helper function to create a mixed array of deterministic and probabilistic causaloids.
// This version contains causaloids that will always evaluate to false.
pub fn get_test_causality_array_mixed_with_failures() -> [BaseCausaloid; 20] {
    let mut causaloids: Vec<BaseCausaloid> = Vec::with_capacity(20);
    // 5 deterministic true
    for _ in 0..5 {
        causaloids.push(get_test_causaloid_deterministic_true());
    }
    // 5 deterministic false
    for _ in 0..5 {
        causaloids.push(get_test_causaloid_deterministic_false());
    }
    // 10 probabilistic
    for _ in 0..10 {
        causaloids.push(get_test_causaloid_probabilistic());
    }
    causaloids.try_into().unwrap()
}

// Helper function for a mixed array where all causaloids can succeed.
pub fn get_test_causality_array_mixed_all_succeed() -> [BaseCausaloid; 15] {
    let mut causaloids: Vec<BaseCausaloid> = Vec::with_capacity(15);
    // 5 deterministic true
    for _ in 0..5 {
        causaloids.push(get_test_causaloid_deterministic_true());
    }
    // 10 probabilistic
    for _ in 0..10 {
        causaloids.push(get_test_causaloid_probabilistic());
    }
    causaloids.try_into().unwrap()
}

// --- Tests for evaluate_deterministic_propagation ---

#[test]
fn test_deterministic_propagation_all() {
    let col = get_test_causality_array_deterministic(true);
    let effect = PropagatingEffect::from_numerical(0.99);
    let res = col.evaluate_deterministic(&effect, &AggregateLogic::All);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    let col = get_test_causality_array_deterministic(false);
    let effect = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_deterministic(&effect, &AggregateLogic::All);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));
}

#[test]
fn test_deterministic_propagation_any() {
    let col = get_test_causality_array_deterministic(true);
    let effect = PropagatingEffect::from_numerical(0.99);
    let res = col.evaluate_deterministic(&effect, &AggregateLogic::Any);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    let col = get_test_causality_array_deterministic(false);
    let effect = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_deterministic(&effect, &AggregateLogic::Any);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));

    // Mixed: one true, rest false
    let mut mixed_col = get_test_causality_array_deterministic(false).to_vec();
    mixed_col[0] = get_test_causaloid_deterministic_true();
    let res = mixed_col.evaluate_deterministic(&effect, &AggregateLogic::Any);
    assert_eq!(res.unwrap(), PropagatingEffect::from_deterministic(true));
}

#[test]
fn test_deterministic_propagation_none() {
    let col = get_test_causality_array_deterministic(true);
    let effect = PropagatingEffect::from_numerical(0.99);
    let res = col.evaluate_deterministic(&effect, &AggregateLogic::None);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));

    let col = get_test_causality_array_deterministic(false);
    let effect = PropagatingEffect::from_numerical(0.1);
    let res = col.evaluate_deterministic(&effect, &AggregateLogic::None);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    // Mixed: one true, rest false
    let mut mixed_col = get_test_causality_array_deterministic(false).to_vec();
    mixed_col[0] = get_test_causaloid_deterministic_true();
    let res = mixed_col.evaluate_deterministic(&effect, &AggregateLogic::None);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));
}

#[test]
fn test_deterministic_propagation_some() {
    let effect = PropagatingEffect::from_numerical(0.99);

    // All true, Some(5) should be true
    let col = get_test_causality_array_deterministic(true);
    let res = col.evaluate_deterministic(&effect, &AggregateLogic::Some(5));
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    // All false, Some(1) should be false
    let col = get_test_causality_array_deterministic(false);
    let res = col.evaluate_deterministic(&effect, &AggregateLogic::Some(1));
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));

    // Mixed: 5 true, 5 false, Some(5) should be true
    let mut mixed_col = get_test_causality_array_deterministic(false).to_vec();

    for item in mixed_col.iter_mut().take(5) {
        *item = get_test_causaloid_deterministic_true();
    }

    let res = mixed_col.evaluate_deterministic(&effect, &AggregateLogic::Some(5));
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    // Mixed: 5 true, 5 false, Some(6) should be false
    let res = mixed_col.evaluate_deterministic(&effect, &AggregateLogic::Some(6));
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));
}

// --- Tests for evaluate_probabilistic_propagation ---

#[test]
fn test_probabilistic_propagation_all() {
    let col = get_test_causality_array_probabilistic();
    let effect = PropagatingEffect::from_numerical(0.99); // Makes all causaloids return 1.0
    let res = col.evaluate_probabilistic(&effect, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(1.0));

    let effect = PropagatingEffect::from_numerical(0.1); // Makes all causaloids return 0.0
    let res = col.evaluate_probabilistic(&effect, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(0.0));

    // Test with specific probabilities
    fn causal_fn_half(_: &PropagatingEffect) ->PropagatingEffect{
        PropagatingEffect::from_probabilistic(0.5)
    }
    let p1 = Causaloid::new(1, causal_fn_half, "p=0.5");

    fn causal_fn_quarter(_: &PropagatingEffect) -> PropagatingEffect {
        PropagatingEffect::from_probabilistic(0.25)
    }
    let p2 = Causaloid::new(2, causal_fn_quarter, "p=0.25");

    let coll: Vec<BaseCausaloid> = vec![p1, p2];
    let effect = PropagatingEffect::from_numerical(0.0);
    let res = coll.evaluate_probabilistic(&effect, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(0.125));
}

#[test]
fn test_probabilistic_propagation_any() {
    let effect_true = PropagatingEffect::from_numerical(0.99); // Causaloid returns 1.0 (true > 0.5)
    let effect_false = PropagatingEffect::from_numerical(0.1); // Causaloid returns 0.0 (false <= 0.5)

    // All false, Any should be false
    let col = get_test_causality_array_probabilistic();
    let res = col.evaluate_probabilistic(&effect_false, &AggregateLogic::Any, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(0.0));

    // All true, Any should be true
    let col = get_test_causality_array_probabilistic();
    let res = col.evaluate_probabilistic(&effect_true, &AggregateLogic::Any, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(1.0));

    // Mixed: one true, rest false, Any should be true
    let mut mixed_col = get_test_causality_array_probabilistic().to_vec();
    mixed_col[0] = get_test_causaloid_deterministic_true(); // Force one to be true
    let res = mixed_col.evaluate_probabilistic(&effect_false, &AggregateLogic::Any, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(1.0));
}

#[test]
fn test_probabilistic_propagation_none() {
    let effect_true = PropagatingEffect::from_numerical(0.99); // Causaloid returns 1.0 (true > 0.5)
    let effect_false = PropagatingEffect::from_numerical(0.1); // Causaloid returns 0.0 (false <= 0.5)

    // All false, None should be true
    let col = get_test_causality_array_probabilistic();
    let res = col.evaluate_probabilistic(&effect_false, &AggregateLogic::None, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(1.0));

    // All true, None should be false
    let col = get_test_causality_array_probabilistic();
    let res = col.evaluate_probabilistic(&effect_true, &AggregateLogic::None, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(0.0));

    // Mixed: one true, rest false, None should be false
    let mut mixed_col = get_test_causality_array_probabilistic().to_vec();
    mixed_col[0] = get_test_causaloid_deterministic_true(); // Force one to be true
    let res = mixed_col.evaluate_probabilistic(&effect_false, &AggregateLogic::None, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(0.0));
}

#[test]
fn test_probabilistic_propagation_some() {
    let effect_true = PropagatingEffect::from_numerical(0.99); // Causaloid returns 1.0 (true > 0.5)
    let effect_false = PropagatingEffect::from_numerical(0.1); // Causaloid returns 0.0 (false <= 0.5)

    // All true, Some(5) should be true
    let col = get_test_causality_array_probabilistic();
    let res = col.evaluate_probabilistic(&effect_true, &AggregateLogic::Some(5), 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(1.0));

    // All false, Some(1) should be false
    let col = get_test_causality_array_probabilistic();
    let res = col.evaluate_probabilistic(&effect_false, &AggregateLogic::Some(1), 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(0.0));

    // Mixed: 5 true, 5 false, Some(5) should be true
    let mut mixed_col = get_test_causality_array_probabilistic().to_vec();
    for item in mixed_col.iter_mut().take(5) {
        *item = get_test_causaloid_deterministic_true();
    }
    let res = mixed_col.evaluate_probabilistic(&effect_false, &AggregateLogic::Some(5), 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(1.0));

    // Mixed: 5 true, 5 false, Some(6) should be false
    let res = mixed_col.evaluate_probabilistic(&effect_false, &AggregateLogic::Some(6), 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_probabilistic(0.0));
}

// --- Tests for evaluate_mixed_propagation ---

#[test]
fn test_mixed_propagation_all_failure() {
    let col = get_test_causality_array_mixed_with_failures();
    let effect = PropagatingEffect::from_numerical(0.55);
    let res = col.evaluate_mixed(&effect, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());

    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));
}

#[test]
fn test_mixed_propagation_all_success() {
    let col = get_test_causality_array_mixed_all_succeed();
    let effect = PropagatingEffect::from_numerical(0.99);
    let res = col.evaluate_mixed(&effect, &AggregateLogic::All, 0.5);
    assert!(res.is_ok());

    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));
}

#[test]
fn test_mixed_propagation_any() {
    let effect_true = PropagatingEffect::from_numerical(0.99); // Causaloid returns true/1.0
    let effect_false = PropagatingEffect::from_numerical(0.1); // Causaloid returns false/0.0

    // All false, Any should be true (due to deterministic true causaloids)
    let col = get_test_causality_array_mixed_all_succeed();
    let res = col.evaluate_mixed(&effect_false, &AggregateLogic::Any, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    // All true, Any should be true
    let col = get_test_causality_array_mixed_all_succeed();
    let res = col.evaluate_mixed(&effect_true, &AggregateLogic::Any, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    // Mixed: 5 true, 5 false, 10 probabilistic (all true with effect_true), Any should be true
    let mut mixed_col = get_test_causality_array_mixed_all_succeed().to_vec();
    // Force one of the initially false deterministic causaloids to be true
    mixed_col[1] = get_test_causaloid_deterministic_true();
    let res = mixed_col.evaluate_mixed(&effect_false, &AggregateLogic::Any, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));
}

#[test]
fn test_mixed_propagation_none() {
    let effect_true = PropagatingEffect::from_numerical(0.99); // Causaloid returns true/1.0
    let effect_false = PropagatingEffect::from_numerical(0.1); // Causaloid returns false/0.0

    // All false, None should be false
    let col = get_test_causality_array_mixed_all_succeed();
    let res = col.evaluate_mixed(&effect_false, &AggregateLogic::None, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));

    // All true, None should be false
    let col = get_test_causality_array_mixed_all_succeed();
    let res = col.evaluate_mixed(&effect_true, &AggregateLogic::None, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));

    // Mixed: one true, rest false, None should be false
    let mut mixed_col = get_test_causality_array_mixed_all_succeed().to_vec();
    mixed_col[0] = get_test_causaloid_deterministic_true(); // Force one to be true
    let res = mixed_col.evaluate_mixed(&effect_false, &AggregateLogic::None, 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));
}

#[test]
fn test_mixed_propagation_some() {
    let effect_true = PropagatingEffect::from_numerical(0.99); // Causaloid returns true/1.0
    let effect_false = PropagatingEffect::from_numerical(0.1); // Causaloid returns false/0.0

    // All true, Some(5) should be true
    let col = get_test_causality_array_mixed_all_succeed();
    let res = col.evaluate_mixed(&effect_true, &AggregateLogic::Some(5), 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    // All false, Some(1) should be true
    let col = get_test_causality_array_mixed_all_succeed();
    let res = col.evaluate_mixed(&effect_false, &AggregateLogic::Some(1), 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    // Mixed: 5 true, 5 false, Some(5) should be true
    let mut mixed_col = get_test_causality_array_mixed_all_succeed().to_vec();
    for item in mixed_col.iter_mut().take(5) {
        *item = get_test_causaloid_deterministic_true();
    }
    let res = mixed_col.evaluate_mixed(&effect_false, &AggregateLogic::Some(5), 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(true));

    // Mixed: 5 true, 5 false, Some(6) should be false
    let res = mixed_col.evaluate_mixed(&effect_false, &AggregateLogic::Some(6), 0.5);
    assert!(res.is_ok());
    let result = res.unwrap();
    assert_eq!(result, PropagatingEffect::from_deterministic(false));
}
