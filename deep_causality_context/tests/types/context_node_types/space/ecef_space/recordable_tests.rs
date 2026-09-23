/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<SpaceRecord>` for EcefSpace. Expected records are the constructor literals under their
//! named fields, all distinct so a swapped field is caught; the wrong-variant refusals name the
//! node.
//!
//! Corner cases (rows A to K): C every other `SpaceRecord` variant refused with the node's
//! identifier, `test_every_other_variant_is_refused`; F/G zero and negative coordinates,
//! `test_zero_and_negative_round_trip`; I a non-finite coordinate, `test_non_finite_round_trips`;
//! K `Float106` narrows and `BFloat16` widens, `test_precision_is_spent_at_the_bound`; every
//! other row n/a.
use deep_causality_context::{EcefSpace, VerticalDatum};
use deep_causality_context_store::{ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::{BFloat16, Float106};

#[test]
fn test_round_trip() {
    let node = EcefSpace::new(3, 1.5, 2.5, 3.5);
    let record = node.to_record().unwrap();
    assert_eq!(
        record,
        SpaceRecord::Ecef {
            x: 1.5,
            y: 2.5,
            z: 3.5
        }
    );
    assert_eq!(EcefSpace::from_record(3, record), Ok(node));
}

#[test]
fn test_every_other_variant_is_refused() {
    let others: [(&str, SpaceRecord); 3] = [
        (
            "Geo",
            SpaceRecord::Geo {
                lat: 1.0,
                lon: 2.0,
                alt: 3.0,
                datum: VerticalDatum::WGS84,
            },
        ),
        (
            "Euclidean",
            SpaceRecord::Euclidean {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        ),
        (
            "Ned",
            SpaceRecord::Ned {
                north: 1.0,
                east: 2.0,
                down: 3.0,
            },
        ),
    ];
    for (found, record) in others {
        assert_eq!(
            EcefSpace::<f64>::from_record(9, record),
            Err(ProjectionError::WrongVariant(9, "Ecef", found))
        );
    }
}

#[test]
fn test_zero_and_negative_round_trip() {
    for node in [
        EcefSpace::new(1, 0.0, 0.0, 0.0),
        EcefSpace::new(1, -1.0, -2.0, -0.5),
    ] {
        assert_eq!(
            EcefSpace::from_record(1, node.to_record().unwrap()),
            Ok(node)
        );
    }
}

#[test]
fn test_non_finite_round_trips() {
    let node = EcefSpace::new(1, 0.0, f64::NAN, 0.0);
    let restored = EcefSpace::<f64>::from_record(1, node.to_record().unwrap()).unwrap();
    assert_ne!(restored, node);
    assert_eq!(restored.to_record().unwrap().kind_name(), "Ecef");
}

#[test]
fn test_precision_is_spent_at_the_bound() {
    // A low half a double cannot hold narrows away; the record is f64 by design.
    let low = 2f64.powi(-70);
    let wide = EcefSpace::new(
        2,
        Float106::new(1.0, low),
        Float106::new(2.0, low),
        Float106::new(3.0, low),
    );
    let restored = EcefSpace::<Float106>::from_record(2, wide.to_record().unwrap()).unwrap();
    assert_eq!(
        restored,
        EcefSpace::new(
            2,
            Float106::new(1.0, 0.0),
            Float106::new(2.0, 0.0),
            Float106::new(3.0, 0.0)
        )
    );
    let narrow = EcefSpace::new(
        2,
        BFloat16::from(1.5),
        BFloat16::from(2.5),
        BFloat16::from(3.5),
    );
    assert_eq!(
        EcefSpace::<BFloat16>::from_record(2, narrow.to_record().unwrap()),
        Ok(narrow)
    );
}
