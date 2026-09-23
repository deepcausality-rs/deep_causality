/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Both variants of `MemorySubstrateErrorEnum` are constructed through their constructors and
//! asserted by variant. Expected values are the literals passed in.
//!
//! Corner cases (rows A to K): A an empty reference in `test_unknown_reference`; F node 0 in
//! `test_reference_refused`; every other row n/a.

use deep_causality_context_store::{MemorySubstrateError, MemorySubstrateErrorEnum, SubstrateRef};
use std::error::Error;

#[test]
fn test_reference_refused() {
    let err = MemorySubstrateError::ReferenceRefused(0);
    assert_eq!(err.kind(), &MemorySubstrateErrorEnum::ReferenceRefused(0));
    assert!(err.to_string().contains("node 0"));
    assert_eq!(
        err,
        MemorySubstrateError::new(MemorySubstrateErrorEnum::ReferenceRefused(0))
    );
}

#[test]
fn test_unknown_reference() {
    let reference = SubstrateRef::new("memory".to_string(), "42".to_string());
    let err = MemorySubstrateError::UnknownReference(reference.clone());
    assert_eq!(
        err.kind(),
        &MemorySubstrateErrorEnum::UnknownReference(reference)
    );
    assert!(err.to_string().contains("memory/42"));
    let empty = MemorySubstrateError::UnknownReference(SubstrateRef::default());
    assert!(empty.to_string().contains("under /"));
}

#[test]
fn test_clone_equality_debug_and_source() {
    let err = MemorySubstrateError::ReferenceRefused(1);
    assert_eq!(err.clone(), err);
    assert_ne!(err, MemorySubstrateError::ReferenceRefused(2));
    assert!(format!("{err:?}").contains("ReferenceRefused"));
    let dyn_err: &dyn Error = &err;
    assert!(dyn_err.source().is_none());
}
