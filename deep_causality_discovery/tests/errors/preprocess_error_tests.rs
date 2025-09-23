/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::PreprocessError;
use std::error::Error;

#[test]
fn test_display() {
    let err = PreprocessError::InvalidColumnIdentifier("column_name".to_string());
    assert_eq!(err.to_string(), "Invalid column identifier: column_name");

    let err = PreprocessError::BinningError("not enough data".to_string());
    assert_eq!(err.to_string(), "Binning error: not enough data");

    let err = PreprocessError::ConfigError("invalid bin count".to_string());
    assert_eq!(
        err.to_string(),
        "Invalid preprocessing configuration: invalid bin count"
    );

    let err = PreprocessError::ImputeError("all NaNs in column".to_string());
    assert_eq!(err.to_string(), "Imputation error: all NaNs in column");
}

#[test]
fn test_source() {
    let err = PreprocessError::InvalidColumnIdentifier("column_name".to_string());
    assert!(err.source().is_none());

    let err = PreprocessError::BinningError("not enough data".to_string());
    assert!(err.source().is_none());

    let err = PreprocessError::ConfigError("invalid bin count".to_string());
    assert!(err.source().is_none());

    let err = PreprocessError::ImputeError("all NaNs in column".to_string());
    assert!(err.source().is_none());
}
