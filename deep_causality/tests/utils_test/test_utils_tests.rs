/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils::*;
use deep_causality::{MonadicCausable, PropagatingEffect};

#[test]
fn test_get_deterministic_test_causality_vec() {
    let vec = get_deterministic_test_causality_vec();
    assert!(!vec.is_empty());
    assert_eq!(vec.len(), 3);
    assert_eq!(vec[0].id(), 1);
    assert_eq!(vec[1].id(), 2);
    assert_eq!(vec[2].id(), 3);
}

#[test]
fn test_get_probabilistic_test_causality_vec() {
    let vec = get_probabilistic_test_causality_vec();
    assert!(!vec.is_empty());
    assert_eq!(vec.len(), 3);
    // Probabilistic causaloids have a fixed ID of 3 in the test_utils
    assert_eq!(vec[0].id(), 3);
    assert_eq!(vec[1].id(), 3);
    assert_eq!(vec[2].id(), 3);
}

#[test]
fn test_get_uncertain_bool_test_causality_vec() {
    let vec = get_uncertain_bool_test_causality_vec();
    assert!(!vec.is_empty());
    assert_eq!(vec.len(), 3);
    // Uncertain bool causaloids have a fixed ID of 3 in the test_utils
    assert_eq!(vec[0].id(), 3);
    assert_eq!(vec[1].id(), 3);
    assert_eq!(vec[2].id(), 3);
}

#[test]
fn test_get_uncertain_float_test_causality_vec() {
    let vec = get_uncertain_float_test_causality_vec();
    assert!(!vec.is_empty());
    assert_eq!(vec.len(), 3);
    // Uncertain float causaloids have a fixed ID of 3 in the test_utils
    assert_eq!(vec[0].id(), 3);
    assert_eq!(vec[1].id(), 3);
    assert_eq!(vec[2].id(), 3);
}

#[test]
fn test_get_test_causaloid_positive_above_threshold() {
    let id = 10;
    let causaloid = get_test_causaloid(id);
    let evidence = 0.60; // Above threshold
    let initial_effect = PropagatingEffect::from_value(evidence);

    let result_effect = causaloid.evaluate(&initial_effect);

    assert!(result_effect.is_ok());
    assert!(result_effect.logs.to_string().contains(&format!(
        "Causaloid {}: Incoming effect: Value({})",
        id, evidence
    )));
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&format!("Processing evidence: {}", evidence))
    );
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&format!("Evidence {} >= threshold 0.55: true", evidence))
    );
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&format!("Causaloid {}: Outgoing effect: Value(true)", id))
    );
}

#[test]
fn test_get_test_causaloid_positive_below_threshold() {
    let id = 11;
    let causaloid = get_test_causaloid(id);
    let evidence = 0.40; // Below threshold
    let initial_effect = PropagatingEffect::from_value(evidence);

    let result_effect = causaloid.evaluate(&initial_effect);

    assert!(result_effect.is_ok());
    assert!(result_effect.logs.to_string().contains(&format!(
        "Causaloid {}: Incoming effect: Value({})",
        id, evidence
    )));
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&format!("Processing evidence: {}", evidence))
    );
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&format!("Evidence {} >= threshold 0.55: false", evidence))
    );
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&format!("Causaloid {}: Outgoing effect: Value(false)", id))
    );
}

#[test]
fn test_get_test_causaloid_negative_evidence() {
    let id = 12;
    let causaloid = get_test_causaloid(id);
    let evidence = -1.0; // Negative evidence
    let initial_effect = PropagatingEffect::from_value(evidence);

    let result_effect = causaloid.evaluate(&initial_effect);
    dbg!(&result_effect);

    assert!(result_effect.is_err());
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&"Causaloid 12: Incoming effect: Value(-1.0)".to_string())
    );
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&"Observation is negative, returning error.".to_string())
    );
}

#[test]
fn test_get_test_causaloid_edge_threshold() {
    let id = 13;
    let causaloid = get_test_causaloid(id);
    let evidence = 0.55; // Exactly at threshold
    let initial_effect = PropagatingEffect::from_value(evidence);

    let result_effect = causaloid.evaluate(&initial_effect);

    assert!(result_effect.is_ok());
    assert!(result_effect.logs.to_string().contains(&format!(
        "Causaloid {}: Incoming effect: Value({})",
        id, evidence
    )));
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&format!("Processing evidence: {}", evidence))
    );
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&format!("Evidence {} >= threshold 0.55: true", evidence))
    );
    assert!(
        result_effect
            .logs
            .to_string()
            .contains(&format!("Causaloid {}: Outgoing effect: Value(true)", id))
    );
}
