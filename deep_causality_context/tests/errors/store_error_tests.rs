/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every variant of `StoreErrorEnum` is constructed through its constructor and asserted by
//! variant; the `Infallible` default is exercised by naming `StoreError<E>` with one parameter.
//! Expected values are the literals passed in.
//!
//! Corner cases (rows A to K): all n/a (an error type has no inputs beyond what it carries).

use deep_causality_context::{StoreError, StoreErrorEnum};
use deep_causality_context_store::{MemoryStorageError, MemorySubstrateError, ProjectionError};
use std::error::Error;

#[test]
fn test_storage_and_projection_with_the_default_substrate() {
    let storage: StoreError<MemoryStorageError> =
        StoreError::Storage(MemoryStorageError::UnknownContext(7));
    assert_eq!(
        storage.kind(),
        &StoreErrorEnum::Storage(MemoryStorageError::UnknownContext(7))
    );
    assert!(storage.to_string().contains("storage"));
    assert!(storage.to_string().contains("container 7"));
    let projection: StoreError<MemoryStorageError> =
        StoreError::Projection(ProjectionError::Version(2, 1));
    assert_eq!(
        projection.kind(),
        &StoreErrorEnum::Projection(ProjectionError::Version(2, 1))
    );
    assert!(projection.to_string().contains("projection"));
    assert!(projection.to_string().contains("version 2"));
}

#[test]
fn test_substrate_variant() {
    let error: StoreError<MemoryStorageError, MemorySubstrateError> =
        StoreError::Substrate(MemorySubstrateError::ReferenceRefused(3));
    assert_eq!(
        error.kind(),
        &StoreErrorEnum::Substrate(MemorySubstrateError::ReferenceRefused(3))
    );
    assert!(error.to_string().contains("substrate"));
    assert!(error.to_string().contains("node 3"));
}

#[test]
fn test_new_from_and_equality() {
    let kind: StoreErrorEnum<MemoryStorageError, MemorySubstrateError> =
        StoreErrorEnum::Projection(ProjectionError::Identity(1, "rule"));
    let built = StoreError::new(kind.clone());
    assert_eq!(built.kind(), &kind);
    let converted: StoreError<MemoryStorageError, MemorySubstrateError> =
        ProjectionError::Identity(1, "rule").into();
    assert_eq!(converted, built);
    assert_ne!(
        converted,
        StoreError::Projection(ProjectionError::Identity(2, "rule"))
    );
    assert_eq!(converted.clone(), converted);
    assert!(format!("{converted:?}").contains("Projection"));
}

#[test]
fn test_is_a_std_error_without_a_source() {
    let error: StoreError<MemoryStorageError> =
        StoreError::Storage(MemoryStorageError::SelfReference(1));
    let dyn_err: &dyn Error = &error;
    assert!(dyn_err.source().is_none());
}
