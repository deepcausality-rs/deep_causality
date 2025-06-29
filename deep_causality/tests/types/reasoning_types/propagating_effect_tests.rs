/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

#[test]
fn test_evidence_deterministic_true() {
    let ev = Evidence::Deterministic(true);
    assert_eq!(ev, Evidence::Deterministic(true));
    assert_eq!(format!("{ev}"), "Deterministic(true)");
}

#[test]
fn test_evidence_deterministic_false() {
    let ev = Evidence::Deterministic(false);
    assert_eq!(ev, Evidence::Deterministic(false));
    assert_eq!(format!("{ev}"), "Deterministic(false)");
}

#[test]
fn test_evidence_numerical() {
    let val = 0.42;
    let ev = Evidence::Numerical(val);
    assert_eq!(ev, Evidence::Numerical(val));
    assert_eq!(format!("{ev}"), "Numerical(0.42)");
}

#[test]
fn test_evidence_probability() {
    let prob = 0.99;
    let ev = Evidence::Probability(prob);
    assert_eq!(ev, Evidence::Probability(prob));
    assert_eq!(format!("{ev}"), "Probability(0.99)");
}
