/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::ReasoningMode;

#[test]
fn test_reasoning_mode_equality() {
    assert_eq!(ReasoningMode::Deterministic, ReasoningMode::Deterministic);
    assert_eq!(ReasoningMode::Probabilistic, ReasoningMode::Probabilistic);
    assert_eq!(ReasoningMode::Symbolic, ReasoningMode::Symbolic);
}

#[test]
fn test_reasoning_mode_inequality() {
    assert_ne!(ReasoningMode::Deterministic, ReasoningMode::Probabilistic);
    assert_ne!(ReasoningMode::Probabilistic, ReasoningMode::Symbolic);
    assert_ne!(ReasoningMode::Symbolic, ReasoningMode::Deterministic);
}

#[test]
fn test_reasoning_mode_clone() {
    let m1 = ReasoningMode::Symbolic;
    let m2 = m1; // Copy
    assert_eq!(m1, m2);
}

#[test]
fn test_reasoning_mode_debug_format() {
    let mode = ReasoningMode::Probabilistic;
    let s = format!("{:?}", mode);
    assert_eq!(s, "Probabilistic");
}

#[test]
fn test_reasoning_mode_hash() {
    use std::collections::HashSet;

    let mut modes = HashSet::new();
    modes.insert(ReasoningMode::Deterministic);
    modes.insert(ReasoningMode::Probabilistic);
    modes.insert(ReasoningMode::Symbolic);

    assert_eq!(modes.len(), 3);
    assert!(modes.contains(&ReasoningMode::Deterministic));
    assert!(modes.contains(&ReasoningMode::Probabilistic));
    assert!(modes.contains(&ReasoningMode::Symbolic));
}
