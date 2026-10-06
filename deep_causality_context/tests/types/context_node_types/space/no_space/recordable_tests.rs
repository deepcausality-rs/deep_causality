/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<SpaceRecord>` for `NoSpace`: writing is `Unrecordable`, reading any variant is
//! `WrongVariant`, both naming the node. Expected values are the identifiers passed in.
//!
//! Corner cases (rows A to K): C every variant of the record refused,
//! `test_reading_any_record_is_refused`; every other row n/a (the type holds no value).
use deep_causality_context::{NoSpace, VerticalDatum};
use deep_causality_context_store::{ProjectionError, Recordable, SpaceRecord};

#[test]
fn test_writing_is_unrecordable() {
    let none: NoSpace<f64> = NoSpace::new();
    assert_eq!(
        none.to_record(),
        Err(ProjectionError::Unrecordable(0, "NoSpace"))
    );
}

#[test]
fn test_reading_any_record_is_refused() {
    let records = [
        SpaceRecord::Geo {
            lat: 1.0,
            lon: 2.0,
            alt: 3.0,
            datum: VerticalDatum::WGS84,
        },
        SpaceRecord::Ecef {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        SpaceRecord::Euclidean {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        SpaceRecord::Ned {
            north: 1.0,
            east: 2.0,
            down: 3.0,
        },
    ];
    for record in records {
        let found = record.kind_name();
        assert_eq!(
            NoSpace::<f64>::from_record(7, record),
            Err(ProjectionError::WrongVariant(7, "NoSpace", found))
        );
    }
}
