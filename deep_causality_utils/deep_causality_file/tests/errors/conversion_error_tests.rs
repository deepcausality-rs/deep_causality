/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_file::ConversionError;
use std::error::Error;

#[test]
fn test_new() {
    let err = ConversionError::new("bad satellite");
    assert_eq!(err.0, "bad satellite");
}

#[test]
fn test_from_str() {
    let err: ConversionError = "from str".into();
    assert_eq!(err.0, "from str");
}

#[test]
fn test_from_string() {
    let err: ConversionError = String::from("from string").into();
    assert_eq!(err.0, "from string");
}

#[test]
fn test_display() {
    let err = ConversionError::new("oops");
    assert_eq!(format!("{err}"), "ConversionError: oops");
}

#[test]
fn test_debug() {
    let err = ConversionError::new("oops");
    assert_eq!(format!("{err:?}"), "ConversionError(\"oops\")");
}

#[test]
fn test_clone_and_eq() {
    let err = ConversionError::new("dup");
    let cloned = err.clone();
    assert_eq!(err, cloned);
    assert_ne!(err, ConversionError::new("other"));
}

#[test]
fn test_is_std_error() {
    let err = ConversionError::new("e");
    let dyn_err: &dyn Error = &err;
    // A ConversionError has no underlying cause.
    assert!(dyn_err.source().is_none());
}
