/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The instrument context: reads by contextoid id return each field, environment readings sit
//! beside their time, and the context round-trips through snapshot and restore.
//!
//! Corner cases: the id layout at tick 0 and at the last tick with an id; a tick recorded twice;
//! a frozen context; a context missing the model or holding another kind under its id; a second
//! payload type through the same alias.

use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, Data, RelationKind, Storable, Temporal,
    TimeScale,
};
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};
use deep_causality_quantum::{
    EnvironmentReading, INSTRUMENT_CONFIGURATION, INSTRUMENT_MODEL, InstrumentContext,
    InterferometerConfiguration, InterferometerContext, InterferometerDatum, InterferometerModel,
    QuantumError, QuantumErrorEnum, WaveVector, environment_at, environment_id,
    environment_time_id, instrument_configuration, instrument_model, interferometer_context,
    record_environment,
};

fn model() -> InterferometerModel<f64> {
    InterferometerModel::new(1.6106e7, 0.08, 0.4, 5.0e-7, 0.5, 1000.0, 30.0).unwrap()
}

fn configuration() -> InterferometerConfiguration<f64> {
    InterferometerConfiguration::new(1.0e-5, 2.0e-6, 1.5708e5)
        .unwrap()
        .with_wave_vector(WaveVector::Down)
        .with_heading(core::f64::consts::PI)
        .unwrap()
        .with_tilt_offset(1.0e-4)
        .unwrap()
        .with_cloud_position([1.0e-3, -2.0e-3, 0.0])
        .unwrap()
        .with_accelerometer_correction(true)
}

fn reading(tide: f64) -> EnvironmentReading<f64> {
    EnvironmentReading::new(tide, 296.15, 4.8e-5).unwrap()
}

/// A gravimeter with readings at ticks 0 and 7200 (seconds), the tide 18.8 µGal apart.
fn gravimeter() -> InterferometerContext<f64> {
    let mut c = interferometer_context(1, "gravimeter", model(), configuration()).unwrap();
    record_environment(&mut c, 0, TimeScale::Second, reading(1.2e-6)).unwrap();
    record_environment(&mut c, 7200, TimeScale::Second, reading(1.012e-6)).unwrap();
    c
}

fn calculation(e: QuantumError) -> String {
    match e.0 {
        QuantumErrorEnum::CalculationError(msg) => msg,
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_the_id_layout() {
    assert_eq!((INSTRUMENT_MODEL, INSTRUMENT_CONFIGURATION), (1, 2));
    assert_eq!(
        (environment_id(0), environment_time_id(0)),
        (Some(3), Some(4))
    );
    assert_eq!(
        (environment_id(1), environment_time_id(1)),
        (Some(5), Some(6))
    );
    // The last tick whose reading has an id, and whose time node has none.
    let last = (1u64 << 63) - 2;
    assert_eq!(environment_id(last), Some(u64::MAX));
    assert_eq!(environment_time_id(last), None);
    assert_eq!(environment_time_id(last - 1), Some(u64::MAX - 1));
    assert_eq!(environment_id(last + 1), None);
}

#[test]
fn test_reads_by_contextoid_id_return_each_field() {
    let c = gravimeter();
    let m = instrument_model(&c).unwrap();
    assert_eq!(
        [
            m.k_eff(),
            m.interrogation_time(),
            m.contrast(),
            m.sensitivity(),
            m.cycle_time(),
            m.white_noise_range(),
            m.setup_time(),
        ],
        [1.6106e7, 0.08, 0.4, 5.0e-7, 0.5, 1000.0, 30.0]
    );
    let k = instrument_configuration(&c).unwrap();
    assert_eq!(k.wave_vector(), WaveVector::Down);
    assert_eq!(k.heading(), core::f64::consts::PI);
    assert_eq!(k.tilt_offset(), 1.0e-4);
    assert_eq!(k.bias_field(), 1.0e-5);
    assert_eq!(k.atom_temperature(), 2.0e-6);
    assert_eq!(k.rabi_frequency(), 1.5708e5);
    assert_eq!(k.cloud_position(), [1.0e-3, -2.0e-3, 0.0]);
    assert!(k.accelerometer_correction());

    // The readers are get_data_by_id under the documented ids.
    assert_eq!(
        c.get_data_by_id(INSTRUMENT_MODEL),
        Some(InterferometerDatum::Model(model()))
    );
    assert_eq!(
        c.get_data_by_id(environment_id(7200).unwrap()),
        Some(InterferometerDatum::Environment(reading(1.012e-6)))
    );
    let e = environment_at(&c, 7200).unwrap();
    assert_eq!(
        (e.earth_tide(), e.temperature(), e.field()),
        (1.012e-6, 296.15, 4.8e-5)
    );
    assert_eq!(environment_at(&c, 0), Some(reading(1.2e-6)));
    assert_eq!(environment_at(&c, 1), None);
    assert_eq!(environment_at(&c, u64::MAX), None);
}

#[test]
fn test_a_reading_sits_beside_its_time() {
    let c = gravimeter();
    let reading_index = c
        .get_node_index_by_id(environment_id(7200).unwrap())
        .unwrap();
    let time_index = c
        .get_node_index_by_id(environment_time_id(7200).unwrap())
        .unwrap();
    let time = c
        .get_node(time_index)
        .unwrap()
        .vertex_type()
        .tempoid()
        .unwrap();
    assert_eq!(time.time_unit(), 7200);
    assert_eq!(time.time_scale(), TimeScale::Second);
    assert_eq!(
        c.get_edge(reading_index, time_index),
        Some(&RelationKind::Temporal)
    );
    assert_eq!(c.number_of_nodes(), 6);
    assert_eq!(c.number_of_edges(), 2);
}

#[test]
fn test_a_tick_recorded_twice_or_without_an_id_is_refused_and_adds_nothing() {
    let mut c = gravimeter();
    let msg =
        calculation(record_environment(&mut c, 7200, TimeScale::Second, reading(0.0)).unwrap_err());
    assert!(msg.contains("refused"), "{msg}");
    assert_eq!(environment_at(&c, 7200), Some(reading(1.012e-6)));

    let msg = calculation(
        record_environment(&mut c, (1u64 << 63) - 2, TimeScale::Second, reading(0.0)).unwrap_err(),
    );
    assert!(msg.contains("u64::MAX"), "{msg}");
    assert_eq!(c.number_of_nodes(), 6);
}

#[test]
fn test_a_reading_whose_time_id_is_taken_adds_nothing_and_a_retry_succeeds() {
    let mut c = interferometer_context(1, "gravimeter", model(), configuration()).unwrap();
    // A node placed by hand under the id the time node of tick 0 needs; the reading's id is free.
    let time_id = environment_time_id(0).unwrap();
    c.add_node(Contextoid::new(
        time_id,
        ContextoidType::Datoid(Data::new(
            time_id,
            InterferometerDatum::Environment(reading(0.0)),
        )),
    ))
    .unwrap();

    let msg =
        calculation(record_environment(&mut c, 0, TimeScale::Second, reading(1.2e-6)).unwrap_err());
    assert!(msg.contains("refused"), "{msg}");
    assert!(msg.contains(&time_id.to_string()), "{msg}");
    assert_eq!(c.get_node_index_by_id(environment_id(0).unwrap()), None);
    assert_eq!((c.number_of_nodes(), c.number_of_edges()), (3, 0));

    // Once the id is free the same call succeeds: nothing of the refused call was left behind.
    c.remove_node(time_id).unwrap();
    record_environment(&mut c, 0, TimeScale::Second, reading(1.2e-6)).unwrap();
    assert_eq!(environment_at(&c, 0), Some(reading(1.2e-6)));
    assert_eq!((c.number_of_nodes(), c.number_of_edges()), (4, 1));
}

#[test]
fn test_a_frozen_context_refuses_a_reading() {
    let mut c = gravimeter();
    c.freeze();
    assert!(record_environment(&mut c, 1, TimeScale::Second, reading(0.0)).is_err());
}

#[test]
fn test_a_missing_or_misplaced_node_is_named() {
    let empty: InterferometerContext<f64> = Context::with_capacity(1, "empty", 1);
    let msg = calculation(instrument_model(&empty).unwrap_err());
    assert!(msg.contains("no model under contextoid id 1"), "{msg}");
    let msg = calculation(instrument_configuration(&empty).unwrap_err());
    assert!(
        msg.contains("no configuration under contextoid id 2"),
        "{msg}"
    );

    // A configuration where the model belongs is not a model, the reverse, and a configuration
    // under a reading's id is not a reading.
    let mut swapped: InterferometerContext<f64> = Context::with_capacity(1, "swapped", 3);
    for (id, datum) in [
        (
            INSTRUMENT_MODEL,
            InterferometerDatum::Configuration(configuration()),
        ),
        (
            INSTRUMENT_CONFIGURATION,
            InterferometerDatum::Model(model()),
        ),
        (
            environment_id(0).unwrap(),
            InterferometerDatum::Configuration(configuration()),
        ),
    ] {
        swapped
            .add_node(Contextoid::new(
                id,
                ContextoidType::Datoid(Data::new(id, datum)),
            ))
            .unwrap();
    }
    assert!(instrument_model(&swapped).is_err());
    assert!(instrument_configuration(&swapped).is_err());
    assert_eq!(environment_at(&swapped, 0), None);
}

#[test]
fn test_the_context_round_trips_through_snapshot_and_restore() {
    let c = gravimeter();
    let snapshot = c.snapshot().unwrap();
    assert_eq!(snapshot.nodes().len(), 6);
    let restored = InterferometerContext::<f64>::restore(snapshot.clone()).unwrap();
    assert_eq!(restored.snapshot().unwrap(), snapshot);
    assert_eq!(instrument_model(&restored).unwrap(), model());
    assert_eq!(
        instrument_configuration(&restored).unwrap(),
        configuration()
    );
    assert_eq!(environment_at(&restored, 0), Some(reading(1.2e-6)));
    assert_eq!(environment_at(&restored, 7200), Some(reading(1.012e-6)));
    let time_index = restored
        .get_node_index_by_id(environment_time_id(7200).unwrap())
        .unwrap();
    let reading_index = restored
        .get_node_index_by_id(environment_id(7200).unwrap())
        .unwrap();
    assert_eq!(
        restored.get_edge(reading_index, time_index),
        Some(&RelationKind::Temporal)
    );
}

/// A qubit's calibration, the payload a QPU stores in the same alias.
#[derive(Debug, Clone, Default, PartialEq)]
struct QubitCalibration {
    t1: f64,
    t2: f64,
    gate_error: f64,
}

impl Storable for QubitCalibration {
    fn to_record(&self) -> DataRecord {
        DataRecord::List(vec![
            self.t1.to_record(),
            self.t2.to_record(),
            self.gate_error.to_record(),
        ])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        match <Vec<f64>>::from_record(id, record)?.as_slice() {
            [t1, t2, gate_error] => Ok(Self {
                t1: *t1,
                t2: *t2,
                gate_error: *gate_error,
            }),
            _ => Err(ProjectionError::MissingField(id, "gate_error")),
        }
    }
}

#[test]
fn test_another_instrument_uses_the_same_alias() {
    let calibration = QubitCalibration {
        t1: 1.2e-4,
        t2: 9.0e-5,
        gate_error: 1.0e-3,
    };
    let mut qpu: InstrumentContext<QubitCalibration, f64> = Context::with_capacity(9, "qpu", 1);
    qpu.add_node(Contextoid::new(
        10,
        ContextoidType::Datoid(Data::new(10, calibration.clone())),
    ))
    .unwrap();
    let restored =
        InstrumentContext::<QubitCalibration, f64>::restore(qpu.snapshot().unwrap()).unwrap();
    assert_eq!(restored.get_data_by_id(10), Some(calibration));
}
