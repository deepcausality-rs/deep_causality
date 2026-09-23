/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructor; no expectation is computed.
//!
//! Corner cases (rows A to K): A empty name in `test_empty_name`; B n/a; C coinciding identifier
//! with a different name in `test_clone_equality_and_debug`; D/E n/a; F identifier 0 in
//! `test_zero_identifier_is_representable` (the record does not refuse it; the storage contract
//! does); G n/a (unsigned); H `u64::MAX` in the same test; I/J/K n/a.

use deep_causality_context_store::ContextRecord;

#[test]
fn test_new_and_getters() {
    let record = ContextRecord::new(7, "weather".to_string());
    assert_eq!(record.id(), 7);
    assert_eq!(record.name(), "weather");
}

#[test]
fn test_empty_name() {
    let record = ContextRecord::new(1, String::new());
    assert_eq!(record.name(), "");
    assert_eq!(record.id(), 1);
}

#[test]
fn test_zero_identifier_is_representable() {
    assert_eq!(ContextRecord::new(0, "z".to_string()).id(), 0);
    assert_eq!(ContextRecord::new(u64::MAX, "m".to_string()).id(), u64::MAX);
}

#[test]
fn test_clone_equality_and_debug() {
    let record = ContextRecord::new(7, "weather".to_string());
    assert_eq!(record.clone(), record);
    assert_ne!(record, ContextRecord::new(8, "weather".to_string()));
    assert_ne!(record, ContextRecord::new(7, "terrain".to_string()));
    assert!(format!("{record:?}").contains("weather"));
}
