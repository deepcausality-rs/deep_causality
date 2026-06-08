/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{BrcdLoadError, CdlError, CpdagError, DataLoadingError};
use std::error::Error;

#[test]
fn test_display_variants() {
    assert!(
        BrcdLoadError::DimensionMismatch("vars".into())
            .to_string()
            .contains("dimension mismatch")
    );
    assert!(
        BrcdLoadError::Tensor("t".into())
            .to_string()
            .contains("tensor construction failed")
    );
}

#[test]
fn test_from_data_loading_error() {
    let e: BrcdLoadError = DataLoadingError::FileNotFound("n.csv".into()).into();
    assert!(matches!(e, BrcdLoadError::DataLoading(_)));
    assert!(e.source().is_some());
}

#[test]
fn test_from_cpdag_error() {
    let e: BrcdLoadError = CpdagError::MissingHeader.into();
    assert!(matches!(e, BrcdLoadError::Cpdag(_)));
    assert!(e.source().is_some());
}

#[test]
fn test_into_cdl_error() {
    let e = CdlError::from(BrcdLoadError::DimensionMismatch("x".into()));
    assert!(matches!(e, CdlError::BrcdLoadError(_)));
}
