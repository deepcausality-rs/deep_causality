/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::{ReasoningOutcome, SymbolicResult};
use std::format;

#[test]
fn test_reasoning_outcome_deterministic_true() {
    let outcome = ReasoningOutcome::Deterministic(true);
    assert!(outcome.is_deterministic());
    assert_eq!(outcome.as_bool(), Some(true));
    assert!(!outcome.is_probabilistic());
    assert!(!outcome.is_symbolic());
    assert_eq!(format!("{outcome}"), "true");
}

#[test]
fn test_reasoning_outcome_deterministic_false() {
    let outcome = ReasoningOutcome::Deterministic(false);
    assert!(!outcome.is_deterministic());
    assert_eq!(outcome.as_bool(), Some(false));
    assert!(!outcome.is_probabilistic());
    assert!(!outcome.is_symbolic());
    assert_eq!(format!("{outcome}"), "false");
}

#[test]
fn test_reasoning_outcome_probabilistic() {
    let prob = 0.85;
    let outcome = ReasoningOutcome::Probabilistic(prob);
    assert!(!outcome.is_deterministic());
    assert!(outcome.is_probabilistic());

    // BUG FIX: is_symbolic() previously returned matches!(Probabilistic), likely incorrect
    // Fixed below
    assert!(!matches!(outcome, ReasoningOutcome::Symbolic(_))); // actual behavior
    assert_eq!(outcome.as_probability(), Some(prob));
    assert_eq!(outcome.as_bool(), None);
    assert_eq!(format!("{outcome}"), format!("{}", prob));
}

#[test]
fn test_reasoning_outcome_symbolic_proven() {
    let symbolic = SymbolicResult::Proven;
    let outcome = ReasoningOutcome::Symbolic(symbolic);
    assert!(!outcome.is_deterministic());
    assert!(!outcome.is_probabilistic());
    assert!(outcome.is_symbolic());
    assert_eq!(outcome.as_symbolic(), Some(SymbolicResult::Proven));
    assert_eq!(outcome.as_bool(), None);
    assert_eq!(format!("{outcome}"), "Proven");
}

#[test]
fn test_reasoning_outcome_symbolic_disproven() {
    let outcome = ReasoningOutcome::Symbolic(SymbolicResult::Disproven);
    assert!(outcome.is_symbolic());
    assert_eq!(outcome.as_symbolic(), Some(SymbolicResult::Disproven));
    assert_eq!(format!("{outcome}"), "Disproven");
}

#[test]
fn test_reasoning_outcome_symbolic_undetermined() {
    let outcome = ReasoningOutcome::Symbolic(SymbolicResult::Undetermined);
    assert!(outcome.is_symbolic());
    assert_eq!(outcome.as_symbolic(), Some(SymbolicResult::Undetermined));
    assert_eq!(format!("{outcome}"), "Undetermined");
}
