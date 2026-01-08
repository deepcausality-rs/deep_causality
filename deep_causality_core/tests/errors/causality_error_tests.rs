/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalityError, CausalityErrorEnum};
use std::collections::HashSet;
use std::fmt::Write;

#[test]
fn test_new() {
    let error_enum = CausalityErrorEnum::Unspecified;
    let error = CausalityError::new(error_enum.clone());
    assert_eq!(error.0, error_enum);
}

#[test]
fn test_display() {
    let error_enum = CausalityErrorEnum::InternalLogicError;
    let error = CausalityError::new(error_enum);
    let mut output = String::new();
    write!(&mut output, "{}", error).unwrap();
    assert_eq!(output, "InternalLogicError");
}

#[test]
fn test_debug() {
    let error_enum = CausalityErrorEnum::TypeConversionError;
    let error = CausalityError::new(error_enum);
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("TypeConversionError"));
    assert!(debug_str.starts_with("CausalityError"));
}

#[test]
fn test_clone() {
    let error1 = CausalityError::new(CausalityErrorEnum::Unspecified);
    let error2 = error1.clone();
    assert_eq!(error1, error2);
}

#[test]
fn test_partial_eq() {
    let error1 = CausalityError::new(CausalityErrorEnum::Unspecified);
    let error2 = CausalityError::new(CausalityErrorEnum::InternalLogicError);
    let error3 = CausalityError::new(CausalityErrorEnum::Unspecified);

    assert_ne!(error1, error2);
    assert_eq!(error1, error3);
}

#[test]
fn test_default() {
    let error: CausalityError = Default::default();
    assert_eq!(error.0, CausalityErrorEnum::default());
}

#[test]
fn test_hash() {
    let error1 = CausalityError::new(CausalityErrorEnum::Unspecified);
    let error2 = CausalityError::new(CausalityErrorEnum::Unspecified);

    let mut set = HashSet::new();
    set.insert(error1);
    // Attempting to insert a duplicate
    assert!(!set.insert(error2));
    assert_eq!(set.len(), 1);

    let error3 = CausalityError::new(CausalityErrorEnum::InternalLogicError);
    assert!(set.insert(error3));
    assert_eq!(set.len(), 2);
}

#[cfg(feature = "std")]
#[test]
fn test_std_error_trait() {
    let error = CausalityError::new(CausalityErrorEnum::Unspecified);
    let _err: Box<dyn std::error::Error> = Box::new(error);
}
