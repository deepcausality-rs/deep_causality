/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::CoordinateError;
use std::error::Error;

#[test]
fn test_coordinate_error_creation() {
    let msg = "invalid index access";
    let err = CoordinateError::new(msg.to_string());
    assert_eq!(err.0, msg);
}

#[test]
fn test_coordinate_error_debug() {
    let msg = "index out of bounds";
    let err = CoordinateError(msg.to_string());
    let dbg = format!("{err:?}");
    let expected = String::from("CoordinateError(\"index out of bounds\")");
    assert_eq!(dbg, expected);
}

#[test]
fn test_coordinate_error_display() {
    let msg = "index out of bounds";
    let err = CoordinateError(msg.to_string());
    let expected = format!("CoordinateError: {msg}");
    assert_eq!(format!("{err}"), expected);
}

#[test]
fn test_coordinate_error_is_error_trait_object() {
    let err = CoordinateError("test error".to_string());
    let trait_obj: &dyn Error = &err;
    assert!(trait_obj.is::<CoordinateError>());
}

#[test]
fn test_coordinate_error_send_sync_static() {
    fn assert_send_sync_static<T: Send + Sync + 'static>(_val: T) {}
    let err = CoordinateError("trait bounds".to_string());
    assert_send_sync_static(err);
}

#[test]
fn test_coordinate_error_equality() {
    let e1 = CoordinateError("equal".to_string());
    let e2 = CoordinateError("equal".to_string());
    assert_eq!(e1.0, e2.0);
}

#[test]
fn test_coordinate_error_from_string() {
    let err: CoordinateError = "converted".to_string().into();
    assert_eq!(err.0, "converted");
}

#[test]
fn test_coordinate_error_from_str() {
    let err: CoordinateError = "converted".into();
    assert_eq!(err.0, "converted");
}
