/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Corner cases: every parameter non-finite and at the edge of its range (zero, negative, the
//! contrast's 1, the cycle at exactly 2T); the record's missing field, wrong kind, and a value
//! the constructor refuses.

use deep_causality_context::Storable;
use deep_causality_context_store::{DataRecord, ProjectionError, ProjectionErrorEnum};
use deep_causality_quantum::{InterferometerModel, QuantumError, QuantumErrorEnum};

const NAMES: [&str; 7] = [
    "k_eff",
    "interrogation_time",
    "contrast",
    "sensitivity",
    "cycle_time",
    "white_noise_range",
    "setup_time",
];

/// Rb at 780 nm, T = 80 ms, contrast 0.4, 50 µGal/√Hz, a 0.5 s cycle, white to 1000 s, 30 s setup.
const NOMINAL: [f64; 7] = [1.6106e7, 0.08, 0.4, 5.0e-7, 0.5, 1000.0, 30.0];

fn build(p: [f64; 7]) -> Result<InterferometerModel<f64>, QuantumError> {
    InterferometerModel::new(p[0], p[1], p[2], p[3], p[4], p[5], p[6])
}

fn with(index: usize, value: f64) -> [f64; 7] {
    let mut p = NOMINAL;
    p[index] = value;
    p
}

fn calculation(e: QuantumError) -> String {
    match e.0 {
        QuantumErrorEnum::CalculationError(msg) => msg,
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_each_parameter_reads_back() {
    let m = build(NOMINAL).unwrap();
    let read = [
        m.k_eff(),
        m.interrogation_time(),
        m.contrast(),
        m.sensitivity(),
        m.cycle_time(),
        m.white_noise_range(),
        m.setup_time(),
    ];
    assert_eq!(read, NOMINAL);
}

#[test]
fn test_a_non_finite_parameter_is_refused_by_name() {
    for (index, name) in NAMES.iter().enumerate() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            match build(with(index, bad)).unwrap_err().0 {
                QuantumErrorEnum::NonFiniteValue(msg) => assert!(msg.contains(name), "{msg}"),
                other => panic!("expected NonFiniteValue for {name}, got {other:?}"),
            }
        }
    }
}

#[test]
fn test_a_parameter_that_must_be_positive_is_refused_at_zero() {
    // k_eff, T, S, the cycle and the white-noise range; the contrast and the setup time have
    // their own ranges below.
    for index in [0usize, 1, 3, 4, 5] {
        let msg = calculation(build(with(index, 0.0)).unwrap_err());
        assert!(msg.contains(NAMES[index]), "{msg}");
        assert!(msg.contains("positive"), "{msg}");
    }
}

#[test]
fn test_the_contrast_lies_in_the_half_open_unit_interval() {
    assert_eq!(build(with(2, 1.0)).unwrap().contrast(), 1.0);
    for bad in [0.0, 1.0 + f64::EPSILON] {
        let msg = calculation(build(with(2, bad)).unwrap_err());
        assert!(msg.contains("(0, 1]"), "{msg}");
    }
}

#[test]
fn test_the_setup_time_may_be_zero_and_not_negative() {
    assert_eq!(build(with(6, 0.0)).unwrap().setup_time(), 0.0);
    let msg = calculation(build(with(6, -1.0e-9)).unwrap_err());
    assert!(msg.contains("setup_time"), "{msg}");
}

#[test]
fn test_the_cycle_spans_at_least_the_three_pulses() {
    // T = 0.08 s, so 2T = 0.16 s is the shortest cycle.
    assert_eq!(build(with(4, 0.16)).unwrap().cycle_time(), 0.16);
    let msg = calculation(build(with(4, 0.159)).unwrap_err());
    assert!(msg.contains("2T"), "{msg}");
}

#[test]
fn test_the_model_round_trips_through_its_record() {
    let m = build(NOMINAL).unwrap();
    let record = m.to_record();
    let DataRecord::Fields(entries) = &record else {
        panic!("expected Fields, got {record:?}");
    };
    let names: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(names, NAMES);
    assert_eq!(InterferometerModel::from_record(1, record), Ok(m));
}

#[test]
fn test_a_record_the_model_cannot_hold_is_refused() {
    let DataRecord::Fields(mut entries) = build(NOMINAL).unwrap().to_record() else {
        panic!("expected Fields");
    };
    entries[2].1 = DataRecord::Number(1.5);
    match InterferometerModel::<f64>::from_record(1, DataRecord::Fields(entries.clone())) {
        Err(ProjectionError(ProjectionErrorEnum::Rejected { id, rule })) => {
            assert_eq!(id, 1);
            assert!(rule.contains("contrast"), "{rule}");
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
    entries.remove(6);
    assert_eq!(
        InterferometerModel::<f64>::from_record(1, DataRecord::Fields(entries)),
        Err(ProjectionError::MissingField(1, "setup_time"))
    );
    assert_eq!(
        InterferometerModel::<f64>::from_record(1, DataRecord::Count(3)),
        Err(ProjectionError::WrongPayload(1, "Fields", "Count"))
    );
}

#[test]
fn test_the_fringe_is_at_mid_fringe_with_the_model_slope() {
    // C k_eff T² / 2 at the nominal model: 0.4 · 1.6106e7 · 0.08² / 2.
    let f = build(NOMINAL).unwrap().fringe();
    assert_eq!(f.operating_point(), 0.5);
    assert!((f.slope() - 0.4 * 1.6106e7 * 0.0064 / 2.0).abs() < 1e-9);
    // The design note's gravimeter: C = 0.5, T = 100 ms; −5 µGal reads 0.498.
    let note = build([1.6106e7, 0.1, 0.5, 2.4e-7, 0.5, 1000.0, 30.0]).unwrap();
    let d = note.fringe().effective_draws(-5.0e-8, 2.4e-7).unwrap();
    assert!((d.probability() - 0.498).abs() < 5.0e-5);
}

#[test]
fn test_one_effective_draw_takes_the_time_the_sensitivity_buys() {
    // The note's gravimeter: C = 0.5, k_eff = 1.6106e7, T = 100 ms, S = 24 µGal/√Hz buys about
    // 2677 draws a second, so a draw takes (C k_eff T² S)² s, within its white-noise range.
    let note = build([1.6106e7, 0.1, 0.5, 2.4e-7, 0.5, 1000.0, 30.0]).unwrap();
    let t = note.instrument_time().unwrap();
    let per_draw = 0.5 * 1.6106e7 * 0.1 * 0.1 * 2.4e-7_f64;
    assert!((t.shot_time() / (per_draw * per_draw) - 1.0).abs() < 1e-12);
    assert_eq!((1.0 / t.shot_time()).round(), 2677.0);
    assert_eq!(t.white_noise_range(), 1000.0);
    // A range shorter than one draw holds none.
    let short = build([1.6106e7, 0.1, 0.5, 2.4e-7, 0.5, 1.0e-6, 30.0]).unwrap();
    assert!(short.instrument_time().is_err());
}
