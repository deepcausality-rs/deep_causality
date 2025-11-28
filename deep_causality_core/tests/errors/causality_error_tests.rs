/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;
use std::collections::HashSet;
use std::fmt::Write;

#[test]
fn test_new() {
    let msg = "Test error message".to_string();
    let error = CausalityError::new(msg.clone());
    assert_eq!(error.0, msg);
}

#[test]
fn test_display() {
    let msg = "Test error message".to_string();
    let error = CausalityError::new(msg.clone());
    let mut output = String::new();
    write!(&mut output, "{}", error).unwrap();
    assert_eq!(output, msg);
}

#[test]
fn test_debug() {
    let msg = "Test error message".to_string();
    let error = CausalityError::new(msg.clone());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains(&msg));
    assert!(debug_str.starts_with("CausalityError"));
}

#[test]
fn test_clone() {
    let msg = "Test error message".to_string();
    let error1 = CausalityError::new(msg);
    let error2 = error1.clone();
    assert_eq!(error1, error2);
}

#[test]
fn test_partial_eq() {
    let msg1 = "Test error message".to_string();
    let error1 = CausalityError::new(msg1);

    let msg2 = "Another error message".to_string();
    let error2 = CausalityError::new(msg2);

    let msg3 = "Test error message".to_string();
    let error3 = CausalityError::new(msg3);

    assert_ne!(error1, error2);
    assert_eq!(error1, error3);
}

#[test]
fn test_default() {
    let error: CausalityError = Default::default();
    assert_eq!(error.0, "");
}

#[test]
fn test_hash() {
    let msg1 = "Test error message".to_string();
    let error1 = CausalityError::new(msg1);

    let msg2 = "Test error message".to_string();
    let error2 = CausalityError::new(msg2);

    let mut set = HashSet::new();
    set.insert(error1);
    // Attempting to insert a duplicate
    assert!(!set.insert(error2));
    assert_eq!(set.len(), 1);

    let msg3 = "Another message".to_string();
    let error3 = CausalityError::new(msg3);
    assert!(set.insert(error3));
    assert_eq!(set.len(), 2);
}

#[cfg(feature = "std")]
#[test]
fn test_std_error_trait() {
    let msg = "Test error".to_string();
    let error = CausalityError::new(msg);
    let _err: Box<dyn std::error::Error> = Box::new(error);
}
