/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<SpaceRecord>` for EuclideanSpace. Expected records are the constructor literals under their
//! named fields, all distinct so a swapped field is caught; the wrong-variant refusals name the
//! node.
//!
//! Corner cases (rows A to K): C every other `SpaceRecord` variant refused with the node's
//! identifier, `test_every_other_variant_is_refused`; F/G zero and negative coordinates,
//! `test_zero_and_negative_round_trip`; I a non-finite coordinate, `test_non_finite_round_trips`;
//! K `Float106` narrows and `BFloat16` widens, `test_precision_is_spent_at_the_bound`; every
//! other row n/a.
use deep_causality_context::{EuclideanSpace, VerticalDatum};
use deep_causality_context_store::{ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::{BFloat16, Float106};

#[test]
fn test_round_trip() {
    let node = EuclideanSpace::new(3, 1.5, 2.5, 3.5);
    let record = node.to_record().unwrap();
    assert_eq!(
        record,
        SpaceRecord::Euclidean {
            x: 1.5,
            y: 2.5,
            z: 3.5
        }
    );
    assert_eq!(EuclideanSpace::from_record(3, record), Ok(node));
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
            "Ecef",
            SpaceRecord::Ecef {
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
            EuclideanSpace::<f64>::from_record(9, record),
            Err(ProjectionError::WrongVariant(9, "Euclidean", found))
        );
    }
}

#[test]
fn test_zero_and_negative_round_trip() {
    let pairs = [
        (
            EuclideanSpace::new(1, 0.0, 0.0, 0.0),
            SpaceRecord::Euclidean {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        (
            EuclideanSpace::new(1, -1.0, -2.0, -0.5),
            SpaceRecord::Euclidean {
                x: -1.0,
                y: -2.0,
                z: -0.5,
            },
        ),
    ];
    for (node, record) in pairs {
        assert_eq!(node.to_record(), Ok(record));
        assert_eq!(EuclideanSpace::from_record(1, record), Ok(node));
    }
}

#[test]
fn test_non_finite_round_trips() {
    let node = EuclideanSpace::new(1, 1.5, f64::NAN, f64::NEG_INFINITY);
    match node.to_record() {
        Ok(SpaceRecord::Euclidean { x, y, z }) => {
            assert_eq!(x, 1.5);
            assert!(y.is_nan());
            assert_eq!(z, f64::NEG_INFINITY);
        }
        other => panic!("expected a Euclidean record, found {other:?}"),
    }
    let record = SpaceRecord::Euclidean {
        x: 1.5,
        y: f64::NAN,
        z: f64::NEG_INFINITY,
    };
    let restored = EuclideanSpace::<f64>::from_record(1, record).unwrap();
    assert_eq!(restored.x(), 1.5);
    assert!(restored.y().is_nan());
    assert_eq!(restored.z(), f64::NEG_INFINITY);
}

#[test]
fn test_precision_is_spent_at_the_bound() {
    // A low half a double cannot hold narrows away; the record is f64 by design.
    let low = 2f64.powi(-70);
    let wide = EuclideanSpace::new(
        2,
        Float106::new(1.0, low),
        Float106::new(2.0, low),
        Float106::new(3.0, low),
    );
    let wide_record = SpaceRecord::Euclidean {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    assert_eq!(wide.to_record(), Ok(wide_record));
    assert_eq!(
        EuclideanSpace::<Float106>::from_record(2, wide_record),
        Ok(EuclideanSpace::new(
            2,
            Float106::new(1.0, 0.0),
            Float106::new(2.0, 0.0),
            Float106::new(3.0, 0.0)
        ))
    );
    let narrow = EuclideanSpace::new(
        2,
        BFloat16::from(1.5),
        BFloat16::from(2.5),
        BFloat16::from(3.5),
    );
    let narrow_record = SpaceRecord::Euclidean {
        x: 1.5,
        y: 2.5,
        z: 3.5,
    };
    assert_eq!(narrow.to_record(), Ok(narrow_record));
    assert_eq!(
        EuclideanSpace::<BFloat16>::from_record(2, narrow_record),
        Ok(narrow)
    );
}
