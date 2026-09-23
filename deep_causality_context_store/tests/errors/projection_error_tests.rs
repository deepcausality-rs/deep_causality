/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every variant of `ProjectionErrorEnum` is constructed through its public constructor and
//! asserted by variant, never by `is_err`. Expected values are the literals passed in; the
//! display text is checked for each literal it must carry.
//!
//! Corner cases (rows A to K): C two errors that differ in one field in
//! `test_clone_equality_and_debug`; F/G/I a scalar of 0, a negative and a non-finite value in
//! `test_scalar_at_zero_negative_and_non_finite`; H `u16::MAX` version in `test_version`; every
//! other row n/a.
use deep_causality_context_store::{ProjectionError, ProjectionErrorEnum};
use std::error::Error;

#[test]
fn test_wrong_variant() {
    let err = ProjectionError::WrongVariant(9, "Euclidean", "Geo");
    assert_eq!(
        err.kind(),
        &ProjectionErrorEnum::WrongVariant {
            id: 9,
            expected: "Euclidean",
            found: "Geo"
        }
    );
    let text = err.to_string();
    assert!(text.contains("node 9"));
    assert!(text.contains("Euclidean"));
    assert!(text.contains("Geo"));
}

#[test]
fn test_wrong_payload() {
    let err = ProjectionError::WrongPayload(4, "Number", "Count");
    assert_eq!(
        err.0,
        ProjectionErrorEnum::WrongPayload {
            id: 4,
            expected: "Number",
            found: "Count"
        }
    );
    let text = err.to_string();
    assert!(text.contains("node 4"));
    assert!(text.contains("Number"));
    assert!(text.contains("Count"));
}

#[test]
fn test_missing_field() {
    let err = ProjectionError::MissingField(9, "lo");
    assert_eq!(
        err.kind(),
        &ProjectionErrorEnum::MissingField { id: 9, field: "lo" }
    );
    let text = err.to_string();
    assert!(text.contains("node 9"));
    assert!(text.contains("lo"));
}

#[test]
fn test_unrecordable() {
    let err = ProjectionError::Unrecordable(2, "NoSpaceTime");
    assert_eq!(
        err.kind(),
        &ProjectionErrorEnum::Unrecordable {
            id: 2,
            kind: "NoSpaceTime"
        }
    );
    let text = err.to_string();
    assert!(text.contains("node 2"));
    assert!(text.contains("NoSpaceTime"));
}

#[test]
fn test_scalar() {
    let err = ProjectionError::Scalar(5, 1.5);
    assert_eq!(
        err.kind(),
        &ProjectionErrorEnum::Scalar { id: 5, value: 1.5 }
    );
    let text = err.to_string();
    assert!(text.contains("node 5"));
    assert!(text.contains("1.5"));
}

#[test]
fn test_identity() {
    let err = ProjectionError::Identity(11, "carried twice");
    assert_eq!(
        err.kind(),
        &ProjectionErrorEnum::Identity {
            id: 11,
            rule: "carried twice"
        }
    );
    let text = err.to_string();
    assert!(text.contains("identifier 11"));
    assert!(text.contains("carried twice"));
}

#[test]
fn test_version() {
    let err = ProjectionError::Version(2, 1);
    assert_eq!(
        err.kind(),
        &ProjectionErrorEnum::Version {
            found: 2,
            supported: 1
        }
    );
    let text = err.to_string();
    assert!(text.contains("version 2"));
    assert!(text.contains("supported 1"));
}

#[test]
fn test_new_wraps_the_kind() {
    let kind = ProjectionErrorEnum::Version {
        found: 3,
        supported: 1,
    };
    let err = ProjectionError::new(kind.clone());
    assert_eq!(err.kind(), &kind);
    assert_eq!(err, ProjectionError::Version(3, 1));
}

#[test]
fn test_clone_equality_and_debug() {
    let err = ProjectionError::Identity(1, "rule");
    let copy = err.clone();
    assert_eq!(err, copy);
    assert_ne!(err, ProjectionError::Identity(2, "rule"));
    assert!(format!("{err:?}").contains("Identity"));
}

#[test]
fn test_is_a_std_error_without_a_source() {
    let err = ProjectionError::Version(2, 1);
    let dyn_err: &dyn Error = &err;
    assert!(dyn_err.source().is_none());
}

#[test]
fn test_scalar_at_zero_negative_and_non_finite() {
    for value in [0.0, -1.0, f64::INFINITY] {
        let err = ProjectionError::Scalar(1, value);
        assert_eq!(err.kind(), &ProjectionErrorEnum::Scalar { id: 1, value });
        assert!(err.to_string().contains(&value.to_string()));
    }
    let nan = ProjectionError::Scalar(1, f64::NAN);
    assert_ne!(nan, nan.clone());
    assert!(nan.to_string().contains("NaN"));
}

#[test]
fn test_version_at_the_top_of_the_range() {
    let err = ProjectionError::Version(u16::MAX, 1);
    assert_eq!(
        err.kind(),
        &ProjectionErrorEnum::Version {
            found: u16::MAX,
            supported: 1
        }
    );
    assert!(err.to_string().contains("65535"));
}
