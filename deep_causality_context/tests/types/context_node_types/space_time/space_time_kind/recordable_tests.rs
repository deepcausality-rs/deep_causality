/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<SpaceTimeRecord>` for `SpaceTimeKind`: total, three variants, three arms.
//! Expected records are the inner constructors' literals.
//!
//! Corner cases (rows A to K): C every variant maps to a distinct record variant,
//! `test_every_variant_round_trips`; every other row is the inner types' and is in their files.
use deep_causality_context::{
    EuclideanSpacetime, LorentzianSpacetime, SpaceTimeKind, TangentSpacetime, TimeScale,
};
use deep_causality_context_store::{Recordable, SpaceTimeRecord};

#[test]
fn test_every_variant_round_trips() {
    let kinds: [SpaceTimeKind<f64>; 3] = [
        SpaceTimeKind::Euclidean(EuclideanSpacetime::new(
            1,
            1.0,
            2.0,
            3.0,
            4.0,
            TimeScale::Second,
        )),
        SpaceTimeKind::Lorentzian(LorentzianSpacetime::new(
            2,
            5.0,
            6.0,
            7.0,
            8.0,
            TimeScale::Nanoseconds,
        )),
        SpaceTimeKind::Tangent(TangentSpacetime::new(
            3, 9.0, 10.0, 11.0, 12.0, 1.0, 0.5, 0.25, 0.125,
        )),
    ];
    let mut names = Vec::new();
    for (id, kind) in (1u64..).zip(kinds) {
        let record = kind.to_record().unwrap();
        names.push(record.kind_name());
        assert_eq!(SpaceTimeKind::from_record(id, record), Ok(kind));
    }
    assert_eq!(names, ["Euclidean", "Lorentzian", "Tangent"]);
}

#[test]
fn test_the_record_lands_in_the_matching_variant() {
    let restored = SpaceTimeKind::<f64>::from_record(
        5,
        SpaceTimeRecord::Lorentzian {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            t: 4.0,
            scale: TimeScale::Second,
        },
    )
    .unwrap();
    assert_eq!(
        restored,
        SpaceTimeKind::Lorentzian(LorentzianSpacetime::new(
            5,
            1.0,
            2.0,
            3.0,
            4.0,
            TimeScale::Second
        ))
    );
}
