/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for [`EvidenceClass`] — the two-valued provenance label on a gate's bound.

use deep_causality_cfd::EvidenceClass;

#[test]
fn test_is_reference_separates_the_two_classes() {
    assert!(EvidenceClass::Reference.is_reference());
    assert!(!EvidenceClass::Tripwire.is_reference());
}

#[test]
fn test_tag_renders_the_lowercase_marker() {
    assert_eq!(EvidenceClass::Reference.tag(), "reference");
    assert_eq!(EvidenceClass::Tripwire.tag(), "tripwire");
}

#[test]
fn test_display_matches_tag() {
    assert_eq!(format!("{}", EvidenceClass::Reference), "reference");
    assert_eq!(format!("{}", EvidenceClass::Tripwire), "tripwire");
}

#[test]
fn test_default_is_tripwire() {
    // The unlabeled case must be the weaker one: claiming agreement with an external reference
    // requires positive evidence, so a bound that declares nothing is a regression tripwire.
    assert_eq!(EvidenceClass::default(), EvidenceClass::Tripwire);
    assert!(!EvidenceClass::default().is_reference());
}

#[test]
fn test_equality_and_copy() {
    let a = EvidenceClass::Reference;
    let b = a; // Copy, not move.
    assert_eq!(a, b);
    assert_ne!(EvidenceClass::Reference, EvidenceClass::Tripwire);
}

#[test]
fn test_debug_is_informative() {
    assert_eq!(format!("{:?}", EvidenceClass::Reference), "Reference");
    assert_eq!(format!("{:?}", EvidenceClass::Tripwire), "Tripwire");
}
