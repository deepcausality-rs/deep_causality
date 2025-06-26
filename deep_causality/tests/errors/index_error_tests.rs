/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::IndexError;
use std::error::Error;

#[test]
fn test_index_error_creation() {
    let msg = "invalid index access";
    let err = IndexError::new(msg.to_string());
    assert_eq!(err.0, msg);
}

#[test]
fn test_index_error_debug() {
    let msg = "index out of bounds";
    let err = IndexError(msg.to_string());
    let dbg = format!("{:?}", err);
    let expected = String::from("IndexError(\"index out of bounds\")");
    assert_eq!(format!("{}", dbg), expected);
}

#[test]
fn test_index_error_display() {
    let msg = "index out of bounds";
    let err = IndexError(msg.to_string());
    let expected = format!("IndexError: {}", msg);
    assert_eq!(format!("{}", err), expected);
}

#[test]
fn test_index_error_is_error_trait_object() {
    let err = IndexError("test error".to_string());
    let trait_obj: &dyn Error = &err;
    assert!(trait_obj.is::<IndexError>());
}

#[test]
fn test_index_error_send_sync_static() {
    fn assert_send_sync_static<T: Send + Sync + 'static>(_val: T) {}
    let err = IndexError("trait bounds".to_string());
    assert_send_sync_static(err);
}

#[test]
fn test_index_error_equality() {
    let e1 = IndexError("equal".to_string());
    let e2 = IndexError("equal".to_string());
    assert_eq!(e1.0, e2.0);
}

#[test]
fn test_index_error_from_string() {
    let err: IndexError = "converted".to_string().into();
    assert_eq!(err.0, "converted");
}

#[test]
fn test_index_error_from_str() {
    let err: IndexError = "converted".into();
    assert_eq!(err.0, "converted");
}
