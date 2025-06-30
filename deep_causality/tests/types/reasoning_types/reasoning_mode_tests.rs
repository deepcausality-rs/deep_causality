/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;
use std::collections::HashSet;

#[test]
fn test_reasoning_mode_display() {
    assert_eq!(ReasoningMode::Deterministic.to_string(), "Deterministic");
    assert_eq!(ReasoningMode::Probabilistic.to_string(), "Probabilistic");
    assert_eq!(ReasoningMode::ContextualLink.to_string(), "ContextualLink");
}

#[test]
fn test_reasoning_mode_debug() {
    assert_eq!(format!("{:?}", ReasoningMode::Deterministic), "Deterministic");
    assert_eq!(format!("{:?}", ReasoningMode::Probabilistic), "Probabilistic");
    assert_eq!(format!("{:?}", ReasoningMode::ContextualLink), "ContextualLink");
}

#[test]
fn test_reasoning_mode_clone() {
    let mode1 = ReasoningMode::Deterministic;
    let mode2 = mode1.clone();
    assert_eq!(mode1, mode2);
}

#[test]
fn test_reasoning_mode_copy() {
    let mode1 = ReasoningMode::Probabilistic;
    let mode2 = mode1; // Copy happens here
    assert_eq!(mode1, mode2);
}

#[test]
fn test_reasoning_mode_partial_eq() {
    assert_eq!(ReasoningMode::Deterministic, ReasoningMode::Deterministic);
    assert_ne!(ReasoningMode::Deterministic, ReasoningMode::Probabilistic);
    assert_ne!(ReasoningMode::Deterministic, ReasoningMode::ContextualLink);

    assert_eq!(ReasoningMode::Probabilistic, ReasoningMode::Probabilistic);
    assert_ne!(ReasoningMode::Probabilistic, ReasoningMode::Deterministic);
    assert_ne!(ReasoningMode::Probabilistic, ReasoningMode::ContextualLink);

    assert_eq!(ReasoningMode::ContextualLink, ReasoningMode::ContextualLink);
    assert_ne!(ReasoningMode::ContextualLink, ReasoningMode::Deterministic);
    assert_ne!(ReasoningMode::ContextualLink, ReasoningMode::Probabilistic);
}

#[test]
fn test_reasoning_mode_hash() {
    let mut set = HashSet::new();
    set.insert(ReasoningMode::Deterministic);
    set.insert(ReasoningMode::Probabilistic);
    set.insert(ReasoningMode::ContextualLink);

    assert!(set.contains(&ReasoningMode::Deterministic));
    assert!(set.contains(&ReasoningMode::Probabilistic));
    assert!(set.contains(&ReasoningMode::ContextualLink));
    assert_eq!(set.len(), 3);

    // Inserting a duplicate should not change the set
    set.insert(ReasoningMode::Deterministic);
    assert_eq!(set.len(), 3);
}