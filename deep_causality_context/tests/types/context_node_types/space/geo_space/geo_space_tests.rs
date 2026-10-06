/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::FloatType;
use deep_causality_context::*;

#[test]
fn test_identifiable_trait() {
    let g = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84).unwrap();
    assert_eq!(g.id(), 1);
}

#[test]
fn test_coordinate_trait() {
    let g = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84).unwrap();

    assert_eq!(g.dimension(), 3);
    assert_eq!(*g.coordinate(0).unwrap(), 52.52);
    assert_eq!(*g.coordinate(1).unwrap(), 13.40);
    assert_eq!(*g.coordinate(2).unwrap(), 34.0);
}

#[test]
fn test_coordinate_out_of_bounds() {
    let g = GeoSpace::new(1, 0.0, 0.0, 0.0, VerticalDatum::WGS84).unwrap();
    let res = g.coordinate(3);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let g = GeoSpace::new(1, 52.520008, 13.404954, 34.0, VerticalDatum::WGS84).unwrap();
    let output = format!("{g}");
    assert!(output.contains("GeoSpace(id=1"));
    assert!(output.contains("lat=52.52"));
    assert!(output.contains("lon=13.40"));
    assert!(output.contains("alt=34.00"));
}

#[test]
fn test_metric_trait() {
    let a = GeoSpace::new(1, 0.0, 0.0, 0.0, VerticalDatum::WGS84).unwrap(); // Equator, Prime Meridian
    let b = GeoSpace::new(2, 0.0, 3.0, 0.0, VerticalDatum::WGS84).unwrap(); // 3° east, same latitude

    // The WGS 84 equator is a circle of radius a = 6 378 137 m (NGA.STND.0036, Table 3.1), so the
    // straight line between two points 3° apart on it is the chord 2a·sin(1.5°).
    let expected: FloatType = 2.0 * 6_378_137.0 * (1.5_f64).to_radians().sin();

    let diff = (a.distance(&b) - expected).abs();
    assert!(diff < 1e-6, "diff = {diff} m");
}

#[test]
fn test_spatial_trait_is_implemented() {
    fn assert_spatial_impl<T: Spatial>() {}
    assert_spatial_impl::<GeoSpace<FloatType>>();
}

#[test]
fn test_equal_altitudes_against_different_datums_are_distinguishable() {
    // Same numeric altitude, different reference. These name different points, so nothing about
    // the type may treat them as the same position.
    let ellipsoidal = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84).unwrap();
    let orthometric = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::EGM2008).unwrap();

    assert_eq!(ellipsoidal.alt(), orthometric.alt());
    assert_ne!(ellipsoidal.datum(), orthometric.datum());
    assert_ne!(ellipsoidal, orthometric);
    assert_ne!(format!("{ellipsoidal}"), format!("{orthometric}"));
}

#[test]
fn test_datum_getter_returns_the_stored_datum() {
    for datum in [
        VerticalDatum::WGS84,
        VerticalDatum::EGM96,
        VerticalDatum::EGM2008,
        VerticalDatum::ISA,
        VerticalDatum::Terrain,
    ] {
        let g = GeoSpace::new(1, 52.52, 13.40, 34.0, datum).unwrap();
        assert_eq!(g.datum(), datum);
    }
}

#[test]
fn test_new_refuses_a_latitude_outside_the_poles() {
    for lat in [90.000_001, -90.000_001, 91.0, -180.0] {
        let result = GeoSpace::new(1, lat, 0.0, 0.0, VerticalDatum::WGS84);
        let err = result.expect_err("a latitude past a pole must be refused");
        assert!(err.to_string().contains("latitude"));
    }
}

#[test]
fn test_new_accepts_the_poles_and_any_finite_longitude() {
    assert!(GeoSpace::new(1, 90.0, 0.0, 0.0, VerticalDatum::WGS84).is_ok());
    assert!(GeoSpace::new(1, -90.0, 0.0, 0.0, VerticalDatum::WGS84).is_ok());
    assert!(GeoSpace::new(1, 0.0, 1_000.0, 0.0, VerticalDatum::WGS84).is_ok());
}

#[test]
fn test_new_refuses_a_coordinate_that_is_not_finite() {
    let bad = [
        (f64::NAN, 0.0, 0.0),
        (0.0, f64::INFINITY, 0.0),
        (0.0, 0.0, f64::NEG_INFINITY),
    ];
    for (lat, lon, alt) in bad {
        let result = GeoSpace::new(1, lat, lon, alt, VerticalDatum::WGS84);
        let err = result.expect_err("a non-finite coordinate must be refused");
        assert!(err.to_string().contains("not finite"));
    }
}
