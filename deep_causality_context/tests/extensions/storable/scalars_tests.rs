/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Storable` for the five primitives. Expected records are the arms the spec assigns; expected
//! values are the literals written.
//!
//! Corner cases (rows A to K): F zero in every type, `test_zero_round_trips`; G negative `f64`,
//! `f32` and `i64`, `test_negative_round_trips`; H `u64::MAX`, `i64::MIN`, `f64::MAX`,
//! `test_boundaries_round_trip`; I `NaN` and infinity, `test_non_finite_round_trips`; J an
//! `f64` beyond `f32` range is `Scalar`, `test_an_f32_that_cannot_hold_the_double_is_refused`;
//! K `f32` and `f64` both covered; the `f32` read rounds (`1.1`), underflows to a signed zero
//! (`±1e-50`), refuses `1e39` and restores an `f32`-written value exactly; every other row n/a.

use deep_causality_context::Storable;
use deep_causality_context_store::{DataRecord, ProjectionError};

#[test]
fn test_each_primitive_writes_its_arm() {
    assert_eq!(1.5f64.to_record(), DataRecord::Number(1.5));
    assert_eq!(1.5f32.to_record(), DataRecord::Number(1.5));
    assert_eq!(7u64.to_record(), DataRecord::Count(7));
    assert_eq!((-7i64).to_record(), DataRecord::Integer(-7));
    assert_eq!(true.to_record(), DataRecord::Flag(true));
}

#[test]
fn test_each_primitive_reads_its_arm() {
    assert_eq!(f64::from_record(1, DataRecord::Number(1.5)), Ok(1.5));
    assert_eq!(f32::from_record(1, DataRecord::Number(1.5)), Ok(1.5f32));
    assert_eq!(u64::from_record(1, DataRecord::Count(7)), Ok(7));
    assert_eq!(i64::from_record(1, DataRecord::Integer(-7)), Ok(-7));
    assert_eq!(bool::from_record(1, DataRecord::Flag(false)), Ok(false));
}

#[test]
fn test_a_wrong_payload_names_the_node_and_both_arms() {
    assert_eq!(
        f64::from_record(4, DataRecord::Count(1)),
        Err(ProjectionError::WrongPayload(4, "Number", "Count"))
    );
    assert_eq!(
        f32::from_record(4, DataRecord::Text("x".to_string())),
        Err(ProjectionError::WrongPayload(4, "Number", "Text"))
    );
    assert_eq!(
        u64::from_record(5, DataRecord::Integer(1)),
        Err(ProjectionError::WrongPayload(5, "Count", "Integer"))
    );
    assert_eq!(
        i64::from_record(6, DataRecord::Count(1)),
        Err(ProjectionError::WrongPayload(6, "Integer", "Count"))
    );
    assert_eq!(
        bool::from_record(7, DataRecord::Number(1.0)),
        Err(ProjectionError::WrongPayload(7, "Flag", "Number"))
    );
}

#[test]
fn test_zero_round_trips() {
    assert_eq!(f64::from_record(1, 0.0f64.to_record()), Ok(0.0));
    assert_eq!(f32::from_record(1, 0.0f32.to_record()), Ok(0.0f32));
    assert_eq!(u64::from_record(1, 0u64.to_record()), Ok(0));
    assert_eq!(i64::from_record(1, 0i64.to_record()), Ok(0));
    assert_eq!(bool::from_record(1, false.to_record()), Ok(false));
}

#[test]
fn test_negative_round_trips() {
    assert_eq!(f64::from_record(1, (-2.5f64).to_record()), Ok(-2.5));
    assert_eq!(f32::from_record(1, (-2.5f32).to_record()), Ok(-2.5f32));
    assert_eq!(i64::from_record(1, (-2i64).to_record()), Ok(-2));
}

#[test]
fn test_boundaries_round_trip() {
    assert_eq!(u64::from_record(1, u64::MAX.to_record()), Ok(u64::MAX));
    assert_eq!(i64::from_record(1, i64::MIN.to_record()), Ok(i64::MIN));
    assert_eq!(f64::from_record(1, f64::MAX.to_record()), Ok(f64::MAX));
    assert_eq!(f32::from_record(1, f32::MAX.to_record()), Ok(f32::MAX));
}

#[test]
fn test_non_finite_round_trips() {
    assert_eq!(
        f64::from_record(1, f64::INFINITY.to_record()),
        Ok(f64::INFINITY)
    );
    assert!(f64::from_record(1, f64::NAN.to_record()).unwrap().is_nan());
    assert_eq!(
        f32::from_record(1, f32::NEG_INFINITY.to_record()),
        Ok(f32::NEG_INFINITY)
    );
}

#[test]
fn test_an_f32_that_cannot_hold_the_double_is_refused() {
    // 1e300 has no f32 value: the widest f32 is about 3.4e38.
    let result = f32::from_record(8, DataRecord::Number(1e300));
    assert_eq!(result, Err(ProjectionError::Scalar(8, 1e300)));
}

#[test]
fn test_an_f32_reads_a_double_as_the_nearest_f32() {
    assert_eq!(f32::from_record(1, DataRecord::Number(1.1)), Ok(1.1f32));
    assert_ne!(f64::from(1.1f32), 1.1);
}

#[test]
fn test_an_f32_reads_a_tiny_double_as_a_signed_zero() {
    let positive = f32::from_record(1, DataRecord::Number(1e-50)).unwrap();
    assert_eq!(positive, 0.0);
    assert!(positive.is_sign_positive());
    let negative = f32::from_record(1, DataRecord::Number(-1e-50)).unwrap();
    assert_eq!(negative, 0.0);
    assert!(negative.is_sign_negative());
}

#[test]
fn test_an_f32_refuses_a_double_just_beyond_its_range() {
    assert_eq!(
        f32::from_record(8, DataRecord::Number(1e39)),
        Err(ProjectionError::Scalar(8, 1e39))
    );
}

#[test]
fn test_an_f32_value_reads_back_exactly() {
    // 0.1f32 is not 0.1: the record holds the f32's exact widening, and reading it gives the f32.
    let value = 0.1f32;
    assert_eq!(value.to_record(), DataRecord::Number(0.10000000149011612));
    assert_eq!(f32::from_record(1, value.to_record()), Ok(0.1f32));
    assert_eq!(
        f32::from_record(1, value.to_record()).map(|v| v.to_bits()),
        Ok(0.1f32.to_bits())
    );
}
