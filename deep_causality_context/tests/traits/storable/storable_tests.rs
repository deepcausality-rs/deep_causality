/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A user struct becomes persistable with one `Storable` implementation, and `Data<_>` of it
//! snapshots and restores through a `Context` with no change to either crate. Expected values
//! are the literals the struct is built from.
//!
//! Corner cases (rows A to K): A an empty `Fields` record read into the struct,
//! `test_a_missing_field_names_the_field`; C a `Fields` record with the right names in another
//! order still reads, `test_field_order_does_not_matter_to_the_reader`; F zero samples and a
//! negative temperature in `test_round_trip_through_a_context`; every other row n/a.

use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, Data, EuclideanSpace,
    EuclideanSpacetime, EuclideanTime, Storable,
};
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

#[derive(Debug, Clone, Default, PartialEq)]
struct SensorReading {
    temperature: f64,
    samples: u64,
    status: String,
}

impl Storable for SensorReading {
    fn to_record(&self) -> DataRecord {
        DataRecord::Fields(vec![
            ("temperature".to_string(), self.temperature.to_record()),
            ("samples".to_string(), self.samples.to_record()),
            ("status".to_string(), self.status.to_record()),
        ])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        let DataRecord::Fields(entries) = record else {
            return Err(ProjectionError::WrongPayload(
                id,
                "Fields",
                record.kind_name(),
            ));
        };
        let field = |name: &'static str| {
            entries
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
                .ok_or(ProjectionError::MissingField(id, name))
        };
        Ok(Self {
            temperature: f64::from_record(id, field("temperature")?)?,
            samples: u64::from_record(id, field("samples")?)?,
            status: String::from_record(id, field("status")?)?,
        })
    }
}

type SensorContext =
    Context<Data<SensorReading>, EuclideanSpace<f64>, EuclideanTime<f64>, EuclideanSpacetime<f64>>;

fn reading() -> SensorReading {
    SensorReading {
        temperature: -3.5,
        samples: 0,
        status: "cold".to_string(),
    }
}

#[test]
fn test_a_struct_round_trips_through_its_record() {
    let record = reading().to_record();
    assert_eq!(record.kind_name(), "Fields");
    assert_eq!(SensorReading::from_record(4, record), Ok(reading()));
}

#[test]
fn test_round_trip_through_a_context() {
    let mut ctx: SensorContext = Context::with_capacity(1, "sensors", 4);
    let node = Contextoid::new(7, ContextoidType::Datoid(Data::new(7, reading())));
    ctx.add_node(node).unwrap();
    let snapshot = ctx.snapshot().unwrap();
    assert_eq!(snapshot.nodes().len(), 1);
    let restored = SensorContext::restore(snapshot.clone()).unwrap();
    assert_eq!(restored.snapshot().unwrap(), snapshot);
    let index = restored.get_node_index_by_id(7).unwrap();
    let held = restored
        .get_node(index)
        .unwrap()
        .vertex_type()
        .dataoid()
        .unwrap();
    assert_eq!(held, &Data::new(7, reading()));
}

#[test]
fn test_a_missing_field_names_the_field() {
    assert_eq!(
        SensorReading::from_record(9, DataRecord::Fields(vec![])),
        Err(ProjectionError::MissingField(9, "temperature"))
    );
    let without_status = DataRecord::Fields(vec![
        ("temperature".to_string(), DataRecord::Number(1.0)),
        ("samples".to_string(), DataRecord::Count(2)),
    ]);
    assert_eq!(
        SensorReading::from_record(9, without_status),
        Err(ProjectionError::MissingField(9, "status"))
    );
}

#[test]
fn test_a_wrong_payload_names_the_node() {
    assert_eq!(
        SensorReading::from_record(3, DataRecord::Count(1)),
        Err(ProjectionError::WrongPayload(3, "Fields", "Count"))
    );
    let wrong_inner = DataRecord::Fields(vec![
        (
            "temperature".to_string(),
            DataRecord::Text("warm".to_string()),
        ),
        ("samples".to_string(), DataRecord::Count(2)),
        ("status".to_string(), DataRecord::Text("ok".to_string())),
    ]);
    assert_eq!(
        SensorReading::from_record(3, wrong_inner),
        Err(ProjectionError::WrongPayload(3, "Number", "Text"))
    );
}

#[test]
fn test_field_order_does_not_matter_to_the_reader() {
    let reordered = DataRecord::Fields(vec![
        ("status".to_string(), DataRecord::Text("cold".to_string())),
        ("samples".to_string(), DataRecord::Count(0)),
        ("temperature".to_string(), DataRecord::Number(-3.5)),
    ]);
    assert_eq!(SensorReading::from_record(1, reordered), Ok(reading()));
}
