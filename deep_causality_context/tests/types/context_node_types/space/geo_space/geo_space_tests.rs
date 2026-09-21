/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::FloatType;
use deep_causality_context::*;

#[test]
fn test_identifiable_trait() {
    let g = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84);
    assert_eq!(g.id(), 1);
}

#[test]
fn test_coordinate_trait() {
    let g = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84);

    assert_eq!(g.dimension(), 3);
    assert_eq!(*g.coordinate(0).unwrap(), 52.52);
    assert_eq!(*g.coordinate(1).unwrap(), 13.40);
    assert_eq!(*g.coordinate(2).unwrap(), 34.0);
}

#[test]
fn test_coordinate_out_of_bounds() {
    let g = GeoSpace::new(1, 0.0, 0.0, 0.0, VerticalDatum::WGS84);
    let res = g.coordinate(3);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let g = GeoSpace::new(1, 52.520008, 13.404954, 34.0, VerticalDatum::WGS84);
    let output = format!("{g}");
    assert!(output.contains("GeoSpace(id=1"));
    assert!(output.contains("lat=52.52"));
    assert!(output.contains("lon=13.40"));
    assert!(output.contains("alt=34.00"));
}

#[test]
fn test_metric_trait() {
    let a = GeoSpace::new(1, 0.0, 0.0, 0.0, VerticalDatum::WGS84); // Equator, Prime Meridian
    let b = GeoSpace::new(2, 0.0, 3.0, 0.0, VerticalDatum::WGS84); // 3° east, same latitude

    let distance = a.distance(&b);

    // Roughly ~333.6 km along the equator for 3° longitude
    let expected: FloatType = 333_584.77995765815;

    let delta = 1e-2; // 1 cm tolerance

    let res = (distance - expected).abs();
    dbg!(res);
    dbg!(delta);
    dbg!(res < delta);

    assert!(
        res < delta,
        "Distance mismatch: expected ~{expected}, got {distance}, diff = {res}"
    );
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
    let ellipsoidal = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84);
    let orthometric = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::EGM2008);

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
        let g = GeoSpace::new(1, 52.52, 13.40, 34.0, datum);
        assert_eq!(g.datum(), datum);
    }
}
