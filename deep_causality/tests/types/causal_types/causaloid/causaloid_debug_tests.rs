/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils;
use deep_causality::{CausaloidRegistry, MonadicCausable, PropagatingEffect};

#[test]
fn test_debug() {
    let causaloid = test_utils::get_test_causaloid_deterministic(1);
    // Before evaluation, is_active returns an error, which the Debug trait should handle.
    let expected_unevaluated = "Causaloid id: 1 
 Causaloid type: Singleton 
 description: tests whether data exceeds threshold of 0.55";
    let actual_unevaluated = format!("{causaloid:?}");
    assert_eq!(actual_unevaluated, expected_unevaluated);

    // Evaluate to active
    let registry = CausaloidRegistry::new();
    let effect = PropagatingEffect::from_numerical(0.99);
    causaloid.evaluate(&registry, &effect);
    let expected_active = "Causaloid id: 1 
 Causaloid type: Singleton 
 description: tests whether data exceeds threshold of 0.55";
    let actual_active = format!("{causaloid:?}");
    assert_eq!(actual_active, expected_active);
}

#[test]
fn test_to_string() {
    let causaloid = test_utils::get_test_causaloid_deterministic(1);
    // Before evaluation, is_active returns an error, which the Display trait should handle.
    let expected_unevaluated = "Causaloid id: 1 
 Causaloid type: Singleton 
 description: tests whether data exceeds threshold of 0.55";
    let actual_unevaluated = causaloid.to_string();
    assert_eq!(actual_unevaluated, expected_unevaluated);

    // Evaluate to active
    let registry = CausaloidRegistry::new();
    let effect = PropagatingEffect::from_numerical(0.99);
    causaloid.evaluate(&registry, &effect);
    let expected_active = "Causaloid id: 1 
 Causaloid type: Singleton 
 description: tests whether data exceeds threshold of 0.55";
    let actual_active = causaloid.to_string();
    assert_eq!(actual_active, expected_active);
}
