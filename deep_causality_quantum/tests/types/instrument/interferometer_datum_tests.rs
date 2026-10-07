/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::Storable;
use deep_causality_context_store::{DataRecord, ProjectionError, ProjectionErrorEnum};
use deep_causality_quantum::{
    EnvironmentReading, InterferometerConfiguration, InterferometerDatum, InterferometerModel,
};

fn model() -> InterferometerModel<f64> {
    InterferometerModel::new(1.6106e7, 0.08, 0.4, 5.0e-7, 0.5, 1000.0, 30.0).unwrap()
}

fn configuration() -> InterferometerConfiguration<f64> {
    InterferometerConfiguration::new(1.0e-5, 2.0e-6, 1.5708e5).unwrap()
}

fn reading() -> EnvironmentReading<f64> {
    EnvironmentReading::new(1.2e-6, 296.15, 4.8e-5).unwrap()
}

fn all() -> [InterferometerDatum<f64>; 3] {
    [
        InterferometerDatum::Model(model()),
        InterferometerDatum::Configuration(configuration()),
        InterferometerDatum::Environment(reading()),
    ]
}

#[test]
fn test_each_accessor_answers_for_its_own_variant_only() {
    let [m, c, e] = all();
    assert_eq!(m.model(), Some(&model()));
    assert_eq!(c.configuration(), Some(&configuration()));
    assert_eq!(e.environment(), Some(&reading()));
    for other in [&c, &e] {
        assert_eq!(other.model(), None);
    }
    for other in [&m, &e] {
        assert_eq!(other.configuration(), None);
    }
    for other in [&m, &c] {
        assert_eq!(other.environment(), None);
    }
}

#[test]
fn test_the_default_is_an_environment_reading_of_zeros() {
    assert_eq!(
        InterferometerDatum::<f64>::default(),
        InterferometerDatum::Environment(EnvironmentReading::default())
    );
}

#[test]
fn test_each_variant_round_trips_under_its_kind() {
    for (datum, kind) in all()
        .into_iter()
        .zip(["model", "configuration", "environment"])
    {
        let record = datum.to_record();
        match &record {
            DataRecord::Fields(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, kind);
            }
            other => panic!("expected Fields, got {other:?}"),
        }
        assert_eq!(InterferometerDatum::from_record(4, record), Ok(datum));
    }
}

fn rule(r: Result<InterferometerDatum<f64>, ProjectionError>) -> String {
    match r {
        Err(ProjectionError(ProjectionErrorEnum::Rejected { id, rule })) => {
            assert_eq!(id, 4);
            rule
        }
        other => panic!("expected Rejected, got {other:?}"),
    }
}

#[test]
fn test_a_record_that_is_not_one_known_entry_is_refused() {
    let empty = DataRecord::Fields(vec![]);
    assert!(rule(InterferometerDatum::from_record(4, empty)).contains("one entry"));

    let entry = |k: &str| (k.to_string(), reading().to_record());
    let two = DataRecord::Fields(vec![entry("environment"), entry("environment")]);
    assert!(rule(InterferometerDatum::from_record(4, two)).contains("one entry"));

    let unknown = DataRecord::Fields(vec![entry("qubit")]);
    assert!(rule(InterferometerDatum::from_record(4, unknown)).contains("'qubit'"));

    assert_eq!(
        InterferometerDatum::<f64>::from_record(4, DataRecord::Flag(false)),
        Err(ProjectionError::WrongPayload(4, "Fields", "Flag"))
    );

    // The variant's own refusal comes through.
    let wrong_inner = DataRecord::Fields(vec![("model".to_string(), reading().to_record())]);
    assert_eq!(
        InterferometerDatum::<f64>::from_record(4, wrong_inner),
        Err(ProjectionError::MissingField(4, "k_eff"))
    );
}
