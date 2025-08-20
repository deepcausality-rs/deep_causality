/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::types::telos_types::teloid_modal::TeloidModal;
use std::cmp::Ordering;

#[test]
fn test_display_format() {
    // Test Obligatory variant
    let obligatory = TeloidModal::Obligatory;
    assert_eq!(format!("{}", obligatory), "Obligatory");

    // Test Impermissible variant
    let impermissible = TeloidModal::Impermissible;
    assert_eq!(format!("{}", impermissible), "Impermissible");

    // Test Optional variant with different costs
    let optional_zero = TeloidModal::Optional(0);
    assert_eq!(format!("{}", optional_zero), "Optional(0)");

    let optional_positive = TeloidModal::Optional(100);
    assert_eq!(format!("{}", optional_positive), "Optional(100)");

    let optional_negative = TeloidModal::Optional(-100);
    assert_eq!(format!("{}", optional_negative), "Optional(-100)");

    // Edge cases for i64
    let optional_max = TeloidModal::Optional(i64::MAX);
    assert_eq!(format!("{}", optional_max), format!("Optional({})", i64::MAX));

    let optional_min = TeloidModal::Optional(i64::MIN);
    assert_eq!(format!("{}", optional_min), format!("Optional({})", i64::MIN));
}

#[test]
fn test_clone_and_copy() {
    // Test Obligatory
    let original_obligatory = TeloidModal::Obligatory;
    let cloned_obligatory = original_obligatory.clone();
    let copied_obligatory = original_obligatory; // Test Copy trait
    assert_eq!(original_obligatory, cloned_obligatory);
    assert_eq!(original_obligatory, copied_obligatory);

    // Test Impermissible
    let original_impermissible = TeloidModal::Impermissible;
    let cloned_impermissible = original_impermissible.clone();
    let copied_impermissible = original_impermissible;
    assert_eq!(original_impermissible, cloned_impermissible);
    assert_eq!(original_impermissible, copied_impermissible);

    // Test Optional
    let original_optional = TeloidModal::Optional(42);
    let cloned_optional = original_optional.clone();
    let copied_optional = original_optional;
    assert_eq!(original_optional, cloned_optional);
    assert_eq!(original_optional, copied_optional);
}

#[test]
fn test_partial_eq() {
    // Equality with self
    assert_eq!(TeloidModal::Obligatory, TeloidModal::Obligatory);
    assert_eq!(TeloidModal::Impermissible, TeloidModal::Impermissible);
    assert_eq!(TeloidModal::Optional(42), TeloidModal::Optional(42));
    assert_eq!(TeloidModal::Optional(-1), TeloidModal::Optional(-1));

    // Inequality between variants
    assert_ne!(TeloidModal::Obligatory, TeloidModal::Impermissible);
    assert_ne!(TeloidModal::Obligatory, TeloidModal::Optional(0));
    assert_ne!(TeloidModal::Impermissible, TeloidModal::Optional(0));

    // Inequality for Optional with different costs
    assert_ne!(TeloidModal::Optional(42), TeloidModal::Optional(43));
    assert_ne!(TeloidModal::Optional(0), TeloidModal::Optional(1));
    assert_ne!(TeloidModal::Optional(-1), TeloidModal::Optional(1));
}

#[test]
fn test_partial_ord() {
    // Test ordering between variants
    assert_eq!(TeloidModal::Obligatory.partial_cmp(&TeloidModal::Impermissible), Some(Ordering::Less));
    assert_eq!(TeloidModal::Impermissible.partial_cmp(&TeloidModal::Obligatory), Some(Ordering::Greater));

    assert_eq!(TeloidModal::Impermissible.partial_cmp(&TeloidModal::Optional(i64::MIN)), Some(Ordering::Less));
    assert_eq!(TeloidModal::Optional(i64::MAX).partial_cmp(&TeloidModal::Impermissible), Some(Ordering::Greater));

    assert_eq!(TeloidModal::Obligatory.partial_cmp(&TeloidModal::Optional(i64::MIN)), Some(Ordering::Less));
    assert_eq!(TeloidModal::Optional(i64::MAX).partial_cmp(&TeloidModal::Obligatory), Some(Ordering::Greater));

    // Test ordering within Optional variant
    assert_eq!(TeloidModal::Optional(10).partial_cmp(&TeloidModal::Optional(20)), Some(Ordering::Less));
    assert_eq!(TeloidModal::Optional(20).partial_cmp(&TeloidModal::Optional(10)), Some(Ordering::Greater));
    assert_eq!(TeloidModal::Optional(10).partial_cmp(&TeloidModal::Optional(10)), Some(Ordering::Equal));
    assert_eq!(TeloidModal::Optional(-10).partial_cmp(&TeloidModal::Optional(10)), Some(Ordering::Less));

    // Edge cases for i64
    assert_eq!(TeloidModal::Optional(i64::MIN).partial_cmp(&TeloidModal::Optional(i64::MAX)), Some(Ordering::Less));
    assert_eq!(TeloidModal::Optional(0).partial_cmp(&TeloidModal::Optional(1)), Some(Ordering::Less));
}

#[test]
fn test_partial_ord_with_operators() {
    // Test ordering between variants
    assert!(TeloidModal::Obligatory < TeloidModal::Impermissible);
    assert!(TeloidModal::Impermissible > TeloidModal::Obligatory);

    assert!(TeloidModal::Impermissible < TeloidModal::Optional(i64::MIN));
    assert!(TeloidModal::Optional(i64::MAX) > TeloidModal::Impermissible);

    assert!(TeloidModal::Obligatory < TeloidModal::Optional(i64::MIN));
    assert!(TeloidModal::Optional(i64::MAX) > TeloidModal::Obligatory);

    // Test ordering within Optional variant
    assert!(TeloidModal::Optional(10) < TeloidModal::Optional(20));
    assert!(TeloidModal::Optional(20) > TeloidModal::Optional(10));
    assert!(TeloidModal::Optional(10) <= TeloidModal::Optional(10));
    assert!(TeloidModal::Optional(10) >= TeloidModal::Optional(10));
    assert!(TeloidModal::Optional(-10) < TeloidModal::Optional(10));

    // Edge cases for i64
    assert!(TeloidModal::Optional(i64::MIN) < TeloidModal::Optional(i64::MAX));
    assert!(TeloidModal::Optional(0) < TeloidModal::Optional(1));
}