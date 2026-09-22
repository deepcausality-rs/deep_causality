/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructors, read back by pattern; no
//! expectation is computed.
//!
//! Corner cases (rows A to K): A/B n/a; C coinciding coordinates across variants in
//! `test_equal_coordinates_under_different_variants`; D/E n/a; F zero and G negative in
//! `test_zero_and_negative_coordinates`; H n/a (no domain bound on a coordinate); I non-finite in
//! `test_non_finite_coordinate`; J/K n/a.

use deep_causality_context_store::{SpaceRecord, VerticalDatum};

fn one_of_each() -> [SpaceRecord; 4] {
    [
        SpaceRecord::Geo {
            lat: 1.0,
            lon: 2.0,
            alt: 3.0,
            datum: VerticalDatum::EGM96,
        },
        SpaceRecord::Ecef {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
        SpaceRecord::Euclidean {
            x: 7.0,
            y: 8.0,
            z: 9.0,
        },
        SpaceRecord::Ned {
            north: 10.0,
            east: 11.0,
            down: 12.0,
        },
    ]
}

#[test]
fn test_four_variants_with_distinct_names() {
    let names: Vec<&str> = one_of_each().iter().map(SpaceRecord::kind_name).collect();
    assert_eq!(names, ["Geo", "Ecef", "Euclidean", "Ned"]);
}

#[test]
fn test_a_position_is_named_not_positional() {
    let record = SpaceRecord::Ned {
        north: 10.0,
        east: 11.0,
        down: 12.0,
    };
    let copy = record;
    let SpaceRecord::Ned { north, east, down } = copy else {
        panic!("a Ned record");
    };
    assert_eq!((north, east, down), (10.0, 11.0, 12.0));
    assert_eq!(record, copy);
}

#[test]
fn test_geo_keeps_its_datum() {
    let [geo, ..] = one_of_each();
    let SpaceRecord::Geo { datum, alt, .. } = geo else {
        panic!("a Geo record");
    };
    assert_eq!(datum, VerticalDatum::EGM96);
    assert_eq!(alt, 3.0);
    assert!(format!("{geo:?}").contains("EGM96"));
}

#[test]
fn test_equal_coordinates_under_different_variants() {
    let ecef = SpaceRecord::Ecef {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    let euclidean = SpaceRecord::Euclidean {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    assert_ne!(ecef, euclidean);
}

#[test]
fn test_zero_and_negative_coordinates() {
    let origin = SpaceRecord::Euclidean {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let below = SpaceRecord::Ned {
        north: -1.0,
        east: 0.0,
        down: -0.5,
    };
    let SpaceRecord::Ned { north, down, .. } = below else {
        panic!("a Ned record");
    };
    assert_eq!(north, -1.0);
    assert_eq!(down, -0.5);
    assert_eq!(origin, origin);
}

#[test]
fn test_non_finite_coordinate() {
    let nan = SpaceRecord::Ecef {
        x: f64::NAN,
        y: 0.0,
        z: 0.0,
    };
    assert_ne!(nan, nan);
    assert_eq!(nan.kind_name(), "Ecef");
}
