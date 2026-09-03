/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Public-surface tests for [`DataLoadingError`]. The `Parse` and `Unknown` representations are
//! constructed through `pub(crate)` helpers, so they are exercised from the loader tests
//! ([`read_sp3_tests`]); here we cover the public `From<std::io::Error>` path, `Display`, `Debug`,
//! and `source`.

use deep_causality_file::DataLoadingError;
use std::error::Error;
use std::io;

#[test]
fn test_from_io_error_display() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "missing file");
    let err = DataLoadingError::from(io_err);
    let msg = format!("{err}");
    assert!(msg.starts_with("data loading: I/O error:"), "got: {msg}");
    assert!(msg.contains("missing file"), "got: {msg}");
}

#[test]
fn test_from_io_error_source_is_some() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let err = DataLoadingError::from(io_err);
    let source = err.source();
    assert!(source.is_some());
    // The wrapped I/O error is the reported cause.
    assert!(source.unwrap().to_string().contains("denied"));
}

#[test]
fn test_debug() {
    let err = DataLoadingError::from(io::Error::other("boom"));
    let dbg = format!("{err:?}");
    assert!(dbg.contains("DataLoadingError"), "got: {dbg}");
}
