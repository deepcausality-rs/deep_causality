/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::FinalizeError;
use std::error::Error;

#[test]
fn test_display() {
    let err = FinalizeError::FormattingError("output format invalid".to_string());
    assert_eq!(err.to_string(), "Formatting error: output format invalid");
}

#[test]
fn test_source() {
    let err = FinalizeError::FormattingError("output format invalid".to_string());
    assert!(err.source().is_none());
}
