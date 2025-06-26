/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

#[test]
fn test_evidence_deterministic_true() {
    let ev = Evidence::Deterministic(true);
    assert_eq!(ev, Evidence::Deterministic(true));
    assert_eq!(format!("{}", ev), "Deterministic(true)");
}

#[test]
fn test_evidence_deterministic_false() {
    let ev = Evidence::Deterministic(false);
    assert_eq!(ev, Evidence::Deterministic(false));
    assert_eq!(format!("{}", ev), "Deterministic(false)");
}

#[test]
fn test_evidence_numerical() {
    let val = 0.42;
    let ev = Evidence::Numerical(val);
    assert_eq!(ev, Evidence::Numerical(val));
    assert_eq!(format!("{}", ev), "Numerical(0.42)");
}

#[test]
fn test_evidence_probability() {
    let prob = 0.99;
    let ev = Evidence::Probability(prob);
    assert_eq!(ev, Evidence::Probability(prob));
    assert_eq!(format!("{}", ev), "Probability(0.99)");
}

#[test]
fn test_evidence_symbolic_atom() {
    let sym = SymbolicRepresentation::new_atom("TempHigh".to_string());
    let ev = Evidence::Symbolic(sym.clone());
    assert_eq!(ev, Evidence::Symbolic(sym.clone()));
    assert_eq!(format!("{}", ev), format!("Symbolic({:?})", sym));
}

#[test]
fn test_evidence_symbolic_expr() {
    let expr = SymbolicRepresentation::new_expr("A ∧ B".to_string());
    let ev = Evidence::Symbolic(expr.clone());
    assert_eq!(ev, Evidence::Symbolic(expr.clone()));
    assert_eq!(format!("{}", ev), format!("Symbolic({:?})", expr));
}
