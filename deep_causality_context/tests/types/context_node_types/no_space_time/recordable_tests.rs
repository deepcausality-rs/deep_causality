/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<SpaceRecord>` and `Recordable<SpaceTimeRecord>` for `NoSpaceTime`: writing is
//! `Unrecordable`, reading any variant is `WrongVariant`, both naming the node. Expected values
//! are the identifiers passed in.
//!
//! Corner cases (rows A to K): C every variant of both records refused,
//! `test_reading_any_record_is_refused`; every other row n/a (the type holds no value).
use deep_causality_context::{NoSpaceTime, TimeScale, VerticalDatum};
use deep_causality_context_store::{ProjectionError, Recordable, SpaceRecord, SpaceTimeRecord};

#[test]
fn test_writing_is_unrecordable() {
    let none: NoSpaceTime<f64> = NoSpaceTime::new();
    assert_eq!(
        Recordable::<SpaceRecord>::to_record(&none),
        Err(ProjectionError::Unrecordable(0, "NoSpaceTime"))
    );
    assert_eq!(
        Recordable::<SpaceTimeRecord>::to_record(&none),
        Err(ProjectionError::Unrecordable(0, "NoSpaceTime"))
    );
}

#[test]
fn test_reading_any_record_is_refused() {
    let spaces = [
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
    for record in spaces {
        let found = record.kind_name();
        assert_eq!(
            <NoSpaceTime<f64> as Recordable<SpaceRecord>>::from_record(7, record),
            Err(ProjectionError::WrongVariant(7, "NoSpaceTime", found))
        );
    }
    let spacetimes = [
        SpaceTimeRecord::Euclidean {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            t: 4.0,
            scale: TimeScale::Second,
        },
        SpaceTimeRecord::Lorentzian {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            t: 4.0,
            scale: TimeScale::Second,
        },
        SpaceTimeRecord::Tangent {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            t: 4.0,
            dt: 1.0,
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
            metric: [[0.0; 4]; 4],
        },
    ];
    for record in spacetimes {
        let found = record.kind_name();
        assert_eq!(
            <NoSpaceTime<f64> as Recordable<SpaceTimeRecord>>::from_record(8, record),
            Err(ProjectionError::WrongVariant(8, "NoSpaceTime", found))
        );
    }
}
