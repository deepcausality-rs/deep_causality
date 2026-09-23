/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<SpaceRecord>` for `SpaceKind`: total, four variants, four arms. Expected records
//! are the inner constructors' literals.
//!
//! Corner cases (rows A to K): C every variant maps to a distinct record variant,
//! `test_every_variant_round_trips`; every other row is the inner types' and is in their files.
use deep_causality_context::{
    EcefSpace, EuclideanSpace, GeoSpace, NedSpace, SpaceKind, VerticalDatum,
};
use deep_causality_context_store::{Recordable, SpaceRecord};

#[test]
fn test_every_variant_round_trips() {
    let kinds: [SpaceKind<f64>; 4] = [
        SpaceKind::Geo(GeoSpace::new(1, 52.5, 13.4, 34.0, VerticalDatum::EGM96)),
        SpaceKind::Ecef(EcefSpace::new(2, 1.0, 2.0, 3.0)),
        SpaceKind::Euclidean(EuclideanSpace::new(3, 4.0, 5.0, 6.0)),
        SpaceKind::Ned(NedSpace::new(4, 7.0, 8.0, 9.0)),
    ];
    let mut names = Vec::new();
    for (id, kind) in (1u64..).zip(kinds) {
        let record = kind.to_record().unwrap();
        names.push(record.kind_name());
        assert_eq!(SpaceKind::from_record(id, record), Ok(kind));
    }
    assert_eq!(names, ["Geo", "Ecef", "Euclidean", "Ned"]);
}

#[test]
fn test_the_record_lands_in_the_matching_variant() {
    let restored = SpaceKind::<f64>::from_record(
        5,
        SpaceRecord::Ned {
            north: 1.0,
            east: 2.0,
            down: 3.0,
        },
    )
    .unwrap();
    assert_eq!(restored, SpaceKind::Ned(NedSpace::new(5, 1.0, 2.0, 3.0)));
}
