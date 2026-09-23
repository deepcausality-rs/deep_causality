/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Storable` for `Float106` and `BFloat16`. `Float106` is two `f64` halves and stores as
//! `Fields { hi, lo }`, so the round trip is exact to every bit; `BFloat16` widens exactly and
//! restores through the rounding its own arithmetic performs. Expected halves are the literals
//! the values are built from.
//!
//! Corner cases (rows A to K): A `Fields` with no entries, `test_a_missing_half_is_named`; F zero
//! in both types, `test_zero_round_trips`; G negative halves,
//! `test_float106_round_trips_to_every_bit`; H a `lo` half at the edge of `f64`'s significand,
//! same test; I `NaN` and infinity in `test_non_finite` and
//! `test_float106_restores_non_finite_halves_as_written`; K both software scalars covered; every
//! other row n/a.

use deep_causality_context::Storable;
use deep_causality_context_store::{DataRecord, ProjectionError};
use deep_causality_num::{BFloat16, Float106};

fn halves(hi: f64, lo: f64) -> DataRecord {
    DataRecord::Fields(vec![
        ("hi".to_string(), DataRecord::Number(hi)),
        ("lo".to_string(), DataRecord::Number(lo)),
    ])
}

#[test]
fn test_float106_round_trips_to_every_bit() {
    // 1 + 2^-70: the low half carries what a double cannot hold.
    let wide = Float106::new(1.0, 2f64.powi(-70));
    assert_eq!(wide.to_record(), halves(1.0, 2f64.powi(-70)));
    assert_eq!(Float106::from_record(1, wide.to_record()), Ok(wide));
    let negative = Float106::new(-3.0, -1e-30);
    assert_eq!(Float106::from_record(1, negative.to_record()), Ok(negative));
    let restored = Float106::from_record(1, negative.to_record()).unwrap();
    assert_eq!(restored.hi(), -3.0);
    assert_eq!(restored.lo(), -1e-30);
}

#[test]
fn test_a_missing_half_is_named() {
    assert_eq!(
        Float106::from_record(9, DataRecord::Fields(vec![])),
        Err(ProjectionError::MissingField(9, "hi"))
    );
    let only_hi = DataRecord::Fields(vec![("hi".to_string(), DataRecord::Number(1.0))]);
    assert_eq!(
        Float106::from_record(9, only_hi),
        Err(ProjectionError::MissingField(9, "lo"))
    );
}

#[test]
fn test_float106_wrong_payloads() {
    assert_eq!(
        Float106::from_record(2, DataRecord::Number(1.0)),
        Err(ProjectionError::WrongPayload(2, "Fields", "Number"))
    );
    let text_half = DataRecord::Fields(vec![
        ("hi".to_string(), DataRecord::Text("1".to_string())),
        ("lo".to_string(), DataRecord::Number(0.0)),
    ]);
    assert_eq!(
        Float106::from_record(2, text_half),
        Err(ProjectionError::WrongPayload(2, "Number", "Text"))
    );
}

#[test]
fn test_bfloat16_widens_exactly_and_restores_rounded() {
    let narrow = BFloat16::from(1.5);
    assert_eq!(narrow.to_record(), DataRecord::Number(1.5));
    assert_eq!(
        BFloat16::from_record(1, DataRecord::Number(1.5)),
        Ok(narrow)
    );
    // 1.001 is not a BFloat16; reading it rounds the way BFloat16::from rounds.
    assert_eq!(
        BFloat16::from_record(1, DataRecord::Number(1.001)),
        Ok(BFloat16::from(1.001))
    );
    assert_eq!(
        BFloat16::from_record(3, DataRecord::Count(1)),
        Err(ProjectionError::WrongPayload(3, "Number", "Count"))
    );
}

#[test]
fn test_zero_round_trips() {
    let zero106 = Float106::new(0.0, 0.0);
    assert_eq!(Float106::from_record(1, zero106.to_record()), Ok(zero106));
    let zero16 = BFloat16::from(0.0);
    assert_eq!(BFloat16::from_record(1, zero16.to_record()), Ok(zero16));
}

#[test]
fn test_non_finite() {
    let inf = BFloat16::from(f64::INFINITY);
    assert_eq!(BFloat16::from_record(1, inf.to_record()), Ok(inf));
    let nan = Float106::new(f64::NAN, 0.0);
    assert!(
        Float106::from_record(1, nan.to_record())
            .unwrap()
            .hi()
            .is_nan()
    );
}

#[test]
fn test_float106_restores_non_finite_halves_as_written() {
    // Normalising (inf, 0) computes inf - inf for the low half, which is NaN.
    let record = halves(f64::INFINITY, 0.0);
    let restored = Float106::from_record(1, record.clone()).unwrap();
    assert_eq!(restored.hi(), f64::INFINITY);
    assert_eq!(restored.lo(), 0.0);
    assert_eq!(restored.to_record(), record);
    let raw = Float106::from_raw(f64::NEG_INFINITY, 0.0);
    assert_eq!(raw.to_record(), halves(f64::NEG_INFINITY, 0.0));
    let restored = Float106::from_record(1, raw.to_record()).unwrap();
    assert_eq!(restored.hi(), f64::NEG_INFINITY);
    assert_eq!(restored.lo(), 0.0);
}
