/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Corner cases: each numeric knob non-finite, the atom temperature at zero and below, the Rabi
//! frequency at zero, a non-finite value in each cloud coordinate; the record's cloud of the wrong
//! length, a missing knob and a knob the setters refuse.

use deep_causality_context::Storable;
use deep_causality_context_store::{DataRecord, ProjectionError, ProjectionErrorEnum};
use deep_causality_quantum::{InterferometerConfiguration, QuantumErrorEnum, WaveVector};

/// 100 mG bias field, 2 µK atoms, a 25 kHz Rabi frequency.
fn nominal() -> InterferometerConfiguration<f64> {
    InterferometerConfiguration::new(1.0e-5, 2.0e-6, 1.5708e5).unwrap()
}

/// Every knob away from its default.
fn turned() -> InterferometerConfiguration<f64> {
    nominal()
        .with_wave_vector(WaveVector::Down)
        .with_heading(core::f64::consts::FRAC_PI_2)
        .unwrap()
        .with_tilt_offset(1.0e-4)
        .unwrap()
        .with_cloud_position([1.0e-3, -2.0e-3, 5.0e-4])
        .unwrap()
        .with_accelerometer_correction(true)
}

#[test]
fn test_new_sets_three_knobs_and_defaults_the_rest() {
    let c = nominal();
    assert_eq!(c.bias_field(), 1.0e-5);
    assert_eq!(c.atom_temperature(), 2.0e-6);
    assert_eq!(c.rabi_frequency(), 1.5708e5);
    assert_eq!(c.wave_vector(), WaveVector::Up);
    assert_eq!(c.heading(), 0.0);
    assert_eq!(c.tilt_offset(), 0.0);
    assert_eq!(c.cloud_position(), [0.0; 3]);
    assert!(!c.accelerometer_correction());
}

#[test]
fn test_each_setter_turns_its_knob() {
    let c = turned()
        .with_bias_field(-3.0e-5)
        .unwrap()
        .with_atom_temperature(5.0e-8)
        .unwrap()
        .with_rabi_frequency(7.0e4)
        .unwrap();
    assert_eq!(c.wave_vector(), WaveVector::Down);
    assert_eq!(c.heading(), core::f64::consts::FRAC_PI_2);
    assert_eq!(c.tilt_offset(), 1.0e-4);
    assert_eq!(c.bias_field(), -3.0e-5);
    assert_eq!(c.atom_temperature(), 5.0e-8);
    assert_eq!(c.rabi_frequency(), 7.0e4);
    assert_eq!(c.cloud_position(), [1.0e-3, -2.0e-3, 5.0e-4]);
    assert!(c.accelerometer_correction());
}

#[test]
fn test_a_non_finite_knob_is_refused_by_name() {
    type Setter =
        fn(
            InterferometerConfiguration<f64>,
            f64,
        )
            -> Result<InterferometerConfiguration<f64>, deep_causality_quantum::QuantumError>;
    let setters: [(&str, Setter); 5] = [
        ("heading", |c, v| c.with_heading(v)),
        ("tilt_offset", |c, v| c.with_tilt_offset(v)),
        ("bias_field", |c, v| c.with_bias_field(v)),
        ("atom_temperature", |c, v| c.with_atom_temperature(v)),
        ("rabi_frequency", |c, v| c.with_rabi_frequency(v)),
    ];
    for (name, set) in setters {
        for bad in [f64::NAN, f64::INFINITY] {
            match set(nominal(), bad).unwrap_err().0 {
                QuantumErrorEnum::NonFiniteValue(msg) => assert!(msg.contains(name), "{msg}"),
                other => panic!("expected NonFiniteValue for {name}, got {other:?}"),
            }
        }
    }
    for axis in 0..3 {
        let mut position = [0.0; 3];
        position[axis] = f64::NAN;
        assert!(matches!(
            nominal().with_cloud_position(position).unwrap_err().0,
            QuantumErrorEnum::NonFiniteValue(_)
        ));
    }
    assert!(InterferometerConfiguration::new(f64::NAN, 2.0e-6, 1.0).is_err());
}

#[test]
fn test_the_atom_temperature_may_be_zero_and_not_negative() {
    assert_eq!(
        nominal()
            .with_atom_temperature(0.0)
            .unwrap()
            .atom_temperature(),
        0.0
    );
    match nominal().with_atom_temperature(-1.0e-9).unwrap_err().0 {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("non-negative"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_the_rabi_frequency_must_be_positive() {
    assert!(nominal().with_rabi_frequency(f64::MIN_POSITIVE).is_ok());
    match nominal().with_rabi_frequency(0.0).unwrap_err().0 {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("positive"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
    assert!(InterferometerConfiguration::new(1.0e-5, 2.0e-6, 0.0).is_err());
}

#[test]
fn test_the_configuration_round_trips_through_its_record() {
    let c = turned();
    assert_eq!(
        InterferometerConfiguration::from_record(2, c.to_record()),
        Ok(c)
    );
}

fn entries_of(c: InterferometerConfiguration<f64>) -> Vec<(String, DataRecord)> {
    match c.to_record() {
        DataRecord::Fields(entries) => entries,
        other => panic!("expected Fields, got {other:?}"),
    }
}

fn set(entries: &mut [(String, DataRecord)], name: &str, value: DataRecord) {
    let entry = entries.iter_mut().find(|(k, _)| k == name).unwrap();
    entry.1 = value;
}

#[test]
fn test_a_record_the_configuration_cannot_hold_is_refused() {
    let mut short = entries_of(turned());
    set(
        &mut short,
        "cloud_position",
        DataRecord::List(vec![DataRecord::Number(0.0); 2]),
    );
    match InterferometerConfiguration::<f64>::from_record(2, DataRecord::Fields(short)) {
        Err(ProjectionError(ProjectionErrorEnum::Rejected { rule, .. })) => {
            assert!(rule.contains("3 coordinates, got 2"), "{rule}")
        }
        other => panic!("expected Rejected, got {other:?}"),
    }

    let mut still = entries_of(turned());
    set(&mut still, "rabi_frequency", DataRecord::Number(0.0));
    match InterferometerConfiguration::<f64>::from_record(2, DataRecord::Fields(still)) {
        Err(ProjectionError(ProjectionErrorEnum::Rejected { rule, .. })) => {
            assert!(rule.contains("rabi_frequency"), "{rule}")
        }
        other => panic!("expected Rejected, got {other:?}"),
    }

    let mut missing = entries_of(turned());
    missing.retain(|(k, _)| k != "accelerometer_correction");
    assert_eq!(
        InterferometerConfiguration::<f64>::from_record(2, DataRecord::Fields(missing)),
        Err(ProjectionError::MissingField(2, "accelerometer_correction"))
    );
}
