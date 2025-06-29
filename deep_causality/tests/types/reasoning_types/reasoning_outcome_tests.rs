/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::PropagatingEffect;
use std::format;

#[test]
fn test_reasoning_outcome_deterministic_true() {
    let effect = PropagatingEffect::Deterministic(true);
    // Reasoning type is deterministic, that is true.
    assert!(effect.is_deterministic());
    // The actual value is true.
    assert_eq!(effect.as_bool(), Some(true));
    assert!(!effect.is_probabilistic());
    assert!(!effect.is_contextual_link());
    assert_eq!(format!("{effect}"), "Deterministic: true");
}

#[test]
fn test_reasoning_outcome_deterministic_false() {
    let effect = PropagatingEffect::Deterministic(false);
    // Resulting effect type is deterministic, that is true.
    assert!(effect.is_deterministic());

    // The actual value is false.
    assert_eq!(effect.as_bool(), Some(false));
    assert!(!effect.is_probabilistic());
    assert!(!effect.is_contextual_link());
    assert_eq!(format!("{effect}"), "Deterministic: false");
}

#[test]
fn test_reasoning_outcome_probabilistic() {
    let prob = 0.85;
    let effect = PropagatingEffect::Probabilistic(prob);
    // Resulting effect type is probabilistic.
    assert!(effect.is_probabilistic());
    assert!(!effect.is_deterministic());

    assert_eq!(effect.as_probability(), Some(prob));
    assert_eq!(effect.as_bool(), None);
    assert_eq!(format!("{effect}"), "Probabilistic: 0.85".to_string());
}
