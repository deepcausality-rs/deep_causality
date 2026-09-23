/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<DataRecord>` for `Data<T: Storable>`: the blanket that plugs every storable
//! payload into the projection. Expected values are the constructor literals.
//!
//! Corner cases (rows A to K): A `Data<Vec<f64>>` over an empty series,
//! `test_a_series_payload_round_trips`; C the identifier comes from the call, not from the
//! record, `test_the_identifier_comes_from_the_call`; every other row is the payloads' and is in
//! `tests/extensions/storable/`.
use deep_causality_context::{Data, SubstrateRef};
use deep_causality_context_store::{DataRecord, ProjectionError, Recordable};

#[test]
fn test_the_three_alias_payloads_round_trip() {
    let number = Data::new(1, 2.5f64);
    assert_eq!(number.to_record(), Ok(DataRecord::Number(2.5)));
    assert_eq!(
        Data::<f64>::from_record(1, DataRecord::Number(2.5)),
        Ok(number)
    );
    let count = Data::new(2, 7u64);
    assert_eq!(count.to_record(), Ok(DataRecord::Count(7)));
    assert_eq!(Data::<u64>::from_record(2, DataRecord::Count(7)), Ok(count));
    let reference = SubstrateRef::new("s".to_string(), "k".to_string());
    let held = Data::new(3, reference.clone());
    assert_eq!(
        held.to_record(),
        Ok(DataRecord::Reference(reference.clone()))
    );
    assert_eq!(
        Data::<SubstrateRef>::from_record(3, DataRecord::Reference(reference)),
        Ok(held)
    );
}

#[test]
fn test_a_series_payload_round_trips() {
    for series in [vec![], vec![0.5, 1.5, 2.5]] {
        let node = Data::new(4, series);
        assert_eq!(
            Data::<Vec<f64>>::from_record(4, node.to_record().unwrap()),
            Ok(node)
        );
    }
}

#[test]
fn test_a_wrong_payload_names_the_node() {
    assert_eq!(
        Data::<f64>::from_record(4, DataRecord::Count(1)),
        Err(ProjectionError::WrongPayload(4, "Number", "Count"))
    );
    assert_eq!(
        Data::<SubstrateRef>::from_record(5, DataRecord::Number(1.0)),
        Err(ProjectionError::WrongPayload(5, "Reference", "Number"))
    );
}

#[test]
fn test_the_identifier_comes_from_the_call() {
    let restored = Data::<u64>::from_record(11, DataRecord::Count(1)).unwrap();
    assert_eq!(restored, Data::new(11, 1));
}
