/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::Storable;
use deep_causality_context_store::{DataRecord, ProjectionError, ProjectionErrorEnum};
use deep_causality_quantum::{EnvironmentReading, QuantumErrorEnum};

/// A 120 µGal tide, 23 °C and 48 µT.
fn reading() -> EnvironmentReading<f64> {
    EnvironmentReading::new(1.2e-6, 296.15, 4.8e-5).unwrap()
}

#[test]
fn test_each_value_reads_back() {
    let r = reading();
    assert_eq!(
        (r.earth_tide(), r.temperature(), r.field()),
        (1.2e-6, 296.15, 4.8e-5)
    );
}

#[test]
fn test_a_non_finite_value_is_refused_by_name() {
    for (index, name) in ["earth_tide", "temperature", "field"].iter().enumerate() {
        let mut v = [1.2e-6, 296.15, 4.8e-5];
        v[index] = f64::NAN;
        match EnvironmentReading::new(v[0], v[1], v[2]).unwrap_err().0 {
            QuantumErrorEnum::NonFiniteValue(msg) => assert!(msg.contains(name), "{msg}"),
            other => panic!("expected NonFiniteValue for {name}, got {other:?}"),
        }
    }
}

#[test]
fn test_the_default_is_a_reading_of_zeros() {
    let r = EnvironmentReading::<f64>::default();
    assert_eq!(
        (r.earth_tide(), r.temperature(), r.field()),
        (0.0, 0.0, 0.0)
    );
}

#[test]
fn test_the_reading_round_trips_and_a_bad_record_is_refused() {
    let r = reading();
    assert_eq!(EnvironmentReading::from_record(3, r.to_record()), Ok(r));

    let nan = DataRecord::Fields(vec![
        ("earth_tide".to_string(), DataRecord::Number(f64::NAN)),
        ("temperature".to_string(), DataRecord::Number(296.15)),
        ("field".to_string(), DataRecord::Number(4.8e-5)),
    ]);
    assert!(matches!(
        EnvironmentReading::<f64>::from_record(3, nan),
        Err(ProjectionError(ProjectionErrorEnum::Rejected { id: 3, .. }))
    ));
    assert_eq!(
        EnvironmentReading::<f64>::from_record(3, DataRecord::Fields(vec![])),
        Err(ProjectionError::MissingField(3, "earth_tide"))
    );
}
