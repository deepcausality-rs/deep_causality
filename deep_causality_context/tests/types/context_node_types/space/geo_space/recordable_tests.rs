/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<SpaceRecord>` for GeoSpace. Expected records are the constructor literals under their
//! named fields, all distinct so a swapped field is caught; the wrong-variant refusals name the
//! node.
//!
//! Corner cases (rows A to K): C every other `SpaceRecord` variant refused with the node's
//! identifier, `test_every_other_variant_is_refused`; F/G zero and negative coordinates,
//! `test_zero_and_negative_round_trip`; I a non-finite or out-of-range coordinate refused,
//! `test_a_record_that_names_no_point_is_refused`;
//! K `Float106` narrows and `BFloat16` widens, `test_precision_is_spent_at_the_bound`; every
//! other row n/a.
use deep_causality_context::{GeoSpace, VerticalDatum};
use deep_causality_context_store::{ProjectionError, Recordable, SpaceRecord};
use deep_causality_num::{BFloat16, Float106};

#[test]
fn test_round_trip() {
    let node = GeoSpace::new(3, 52.5, 13.4, 34.0, VerticalDatum::EGM96).unwrap();
    let record = node.to_record().unwrap();
    assert_eq!(
        record,
        SpaceRecord::Geo {
            lat: 52.5,
            lon: 13.4,
            alt: 34.0,
            datum: VerticalDatum::EGM96
        }
    );
    assert_eq!(GeoSpace::from_record(3, record), Ok(node));
}

#[test]
fn test_every_other_variant_is_refused() {
    let others: [(&str, SpaceRecord); 3] = [
        (
            "Ecef",
            SpaceRecord::Ecef {
                x: 1.0,
                y: 2.0,
                z: 3.0,
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
            GeoSpace::<f64>::from_record(9, record),
            Err(ProjectionError::WrongVariant(9, "Geo", found))
        );
    }
}

#[test]
fn test_zero_and_negative_round_trip() {
    let pairs = [
        (
            GeoSpace::new(1, 0.0, 0.0, 0.0, VerticalDatum::WGS84).unwrap(),
            SpaceRecord::Geo {
                lat: 0.0,
                lon: 0.0,
                alt: 0.0,
                datum: VerticalDatum::WGS84,
            },
        ),
        (
            GeoSpace::new(1, -33.9, -70.6, -12.0, VerticalDatum::Terrain).unwrap(),
            SpaceRecord::Geo {
                lat: -33.9,
                lon: -70.6,
                alt: -12.0,
                datum: VerticalDatum::Terrain,
            },
        ),
    ];
    for (node, record) in pairs {
        assert_eq!(node.to_record(), Ok(record));
        assert_eq!(GeoSpace::from_record(1, record), Ok(node));
    }
}

#[test]
fn test_a_record_that_names_no_point_is_refused() {
    // A latitude past a pole, or a coordinate that is not finite, names no point on the ellipsoid,
    // so the restore refuses the record rather than build a node the constructor would refuse.
    let records = [
        (91.0, 13.4, 0.0),
        (-90.5, 13.4, 0.0),
        (f64::NAN, 13.4, 0.0),
        (52.5, f64::INFINITY, 0.0),
        (52.5, 13.4, f64::NEG_INFINITY),
    ];
    for (lat, lon, alt) in records {
        let record = SpaceRecord::Geo {
            lat,
            lon,
            alt,
            datum: VerticalDatum::WGS84,
        };
        match GeoSpace::<f64>::from_record(4, record) {
            Err(e) => assert!(
                matches!(
                    e.kind(),
                    deep_causality_context_store::ProjectionErrorEnum::Rejected { id: 4, .. }
                ),
                "{e}"
            ),
            Ok(node) => panic!("restored a node that names no point: {node:?}"),
        }
    }
}

#[test]
fn test_precision_is_spent_at_the_bound() {
    // A low half a double cannot hold narrows away; the record is f64 by design.
    let low = 2f64.powi(-70);
    let wide = GeoSpace::new(
        2,
        Float106::new(1.0, low),
        Float106::new(2.0, low),
        Float106::new(3.0, low),
        VerticalDatum::ISA,
    )
    .unwrap();
    let wide_record = SpaceRecord::Geo {
        lat: 1.0,
        lon: 2.0,
        alt: 3.0,
        datum: VerticalDatum::ISA,
    };
    assert_eq!(wide.to_record(), Ok(wide_record));
    assert_eq!(
        GeoSpace::<Float106>::from_record(2, wide_record),
        Ok(GeoSpace::new(
            2,
            Float106::new(1.0, 0.0),
            Float106::new(2.0, 0.0),
            Float106::new(3.0, 0.0),
            VerticalDatum::ISA
        )
        .unwrap())
    );
    let narrow = GeoSpace::new(
        2,
        BFloat16::from(1.5),
        BFloat16::from(2.5),
        BFloat16::from(3.5),
        VerticalDatum::EGM2008,
    )
    .unwrap();
    let narrow_record = SpaceRecord::Geo {
        lat: 1.5,
        lon: 2.5,
        alt: 3.5,
        datum: VerticalDatum::EGM2008,
    };
    assert_eq!(narrow.to_record(), Ok(narrow_record));
    assert_eq!(
        GeoSpace::<BFloat16>::from_record(2, narrow_record),
        Ok(narrow)
    );
}
