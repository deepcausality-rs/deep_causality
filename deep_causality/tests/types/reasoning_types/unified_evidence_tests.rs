/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

#[test]
fn test_evidence_display() {
    let evidence1 = Evidence::Deterministic(true);
    assert_eq!(format!("{evidence1}"), "Deterministic(true)");

    let evidence2 = Evidence::Numerical(42.0);
    assert_eq!(format!("{evidence2}"), "Numerical(42.0)");

    let evidence3 = Evidence::Probability(0.75);
    assert_eq!(format!("{evidence3}"), "Probability(0.75)");

    let evidence4 = Evidence::ContextualLink(1, 2);
    assert_eq!(format!("{evidence4}"), "ContextualLink(1, 2)");
}

#[test]
fn test_evidence_debug() {
    let evidence1 = Evidence::Deterministic(true);
    assert_eq!(format!("{evidence1:?}"), "Deterministic(true)");

    let evidence2 = Evidence::Numerical(42.0);
    assert_eq!(format!("{evidence2:?}"), "Numerical(42.0)");

    let evidence3 = Evidence::Probability(0.75);
    assert_eq!(format!("{evidence3:?}"), "Probability(0.75)");

    let evidence4 = Evidence::ContextualLink(1, 2);
    assert_eq!(format!("{evidence4:?}"), "ContextualLink(1, 2)");
}

#[test]
fn test_evidence_clone() {
    let evidence1 = Evidence::Deterministic(true);
    let clone1 = evidence1.clone();
    assert_eq!(evidence1, clone1);

    let evidence2 = Evidence::Numerical(42.0);
    let clone2 = evidence2.clone();
    assert_eq!(evidence2, clone2);

    let evidence3 = Evidence::Probability(0.75);
    let clone3 = evidence3.clone();
    assert_eq!(evidence3, clone3);

    let evidence4 = Evidence::ContextualLink(1, 2);
    let clone4 = evidence4.clone();
    assert_eq!(evidence4, clone4);
}

#[test]
fn test_evidence_partial_eq() {
    let evidence1 = Evidence::Deterministic(true);
    let evidence2 = Evidence::Deterministic(true);
    let evidence3 = Evidence::Deterministic(false);
    assert_eq!(evidence1, evidence2);
    assert_ne!(evidence1, evidence3);

    let evidence4 = Evidence::Numerical(42.0);
    let evidence5 = Evidence::Numerical(42.0);
    let evidence6 = Evidence::Numerical(43.0);
    assert_eq!(evidence4, evidence5);
    assert_ne!(evidence4, evidence6);

    let evidence7 = Evidence::Probability(0.75);
    let evidence8 = Evidence::Probability(0.75);
    let evidence9 = Evidence::Probability(0.76);
    assert_eq!(evidence7, evidence8);
    assert_ne!(evidence7, evidence9);

    let evidence10 = Evidence::ContextualLink(1, 2);
    let evidence11 = Evidence::ContextualLink(1, 2);
    let evidence12 = Evidence::ContextualLink(2, 1);
    assert_eq!(evidence10, evidence11);
    assert_ne!(evidence10, evidence12);

    assert_ne!(evidence1, evidence4);
    assert_ne!(evidence1, evidence7);
    assert_ne!(evidence1, evidence10);
    assert_ne!(evidence4, evidence7);
    assert_ne!(evidence4, evidence10);
    assert_ne!(evidence7, evidence10);
}
