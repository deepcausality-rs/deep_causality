/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `GeoSpace::distance` is the straight-line distance between the Earth-centred Cartesian
//! positions of two points (IOGP 373-7-2, §2.2.1).
//!
//! Provenance of the expected values:
//! - `A` and `B`: WGS 84 semi-major and semi-minor axes, NGA.STND.0036 Tables 3.1 and 3.5.
//! - The North Sea point and its geocentric coordinates: the worked example of IOGP 373-7-2,
//!   §2.2.1, p. 90, published to the millimetre.
//!
//! Corner cases (rows A to K): A/B n/a; C coinciding points under two longitudes 360° apart,
//! `test_longitudes_360_degrees_apart_name_the_same_point`; D n/a; E the poles and the equator,
//! where the ellipsoid's two axes are read off directly, `test_pole_to_pole_is_twice_the_semi_minor_axis`
//! and `test_equatorial_antipodes_are_twice_the_semi_major_axis`; F zero distance in the same
//! point test; G negative latitude in the pole test; H n/a; I NaN for datums that are not heights
//! above the ellipsoid, `test_a_datum_other_than_wgs84_has_no_distance`; J/K n/a.

use deep_causality_context::*;

const A: f64 = 6_378_137.0;
const B: f64 = 6_356_752.314_2;

fn at(lat: f64, lon: f64, alt: f64) -> GeoSpace<f64> {
    GeoSpace::new(1, lat, lon, alt, VerticalDatum::WGS84).unwrap()
}

#[test]
fn test_the_iogp_worked_example() {
    // 53°48'33.820"N, 2°07'46.380"E, h = 73.0 m is X = 3 771 793.968 m, Y = 140 253.342 m,
    // Z = 5 124 304.349 m (IOGP 373-7-2, §2.2.1). The point (0°, 0°, 0 m) is (a, 0, 0).
    let lat = 53.0 + 48.0 / 60.0 + 33.820 / 3600.0;
    let lon = 2.0 + 7.0 / 60.0 + 46.380 / 3600.0;
    let north_sea = at(lat, lon, 73.0);
    let origin = at(0.0, 0.0, 0.0);

    let (x, y, z) = (3_771_793.968 - A, 140_253.342, 5_124_304.349);
    let expected = (x * x + y * y + z * z).sqrt();

    // The published coordinates are rounded to the millimetre.
    let diff = (north_sea.distance(&origin) - expected).abs();
    assert!(diff < 0.002, "diff = {diff} m");
}

#[test]
fn test_pole_to_pole_is_twice_the_semi_minor_axis() {
    let north = at(90.0, 0.0, 0.0);
    let south = at(-90.0, 0.0, 0.0);
    let diff = (north.distance(&south) - 2.0 * B).abs();
    assert!(diff < 0.001, "diff = {diff} m");
}

#[test]
fn test_equatorial_antipodes_are_twice_the_semi_major_axis() {
    let a = at(0.0, 0.0, 0.0);
    let b = at(0.0, 180.0, 0.0);
    let diff = (a.distance(&b) - 2.0 * A).abs();
    assert!(diff < 1e-6, "diff = {diff} m");
}

#[test]
fn test_pole_to_equator_is_the_ellipse_chord() {
    // (0, 0, b) to (a, 0, 0). A sphere of any one radius gives a different value.
    let pole = at(90.0, 0.0, 0.0);
    let equator = at(0.0, 0.0, 0.0);
    let diff = (pole.distance(&equator) - (A * A + B * B).sqrt()).abs();
    assert!(diff < 0.001, "diff = {diff} m");
}

#[test]
fn test_a_change_of_ellipsoidal_height_alone_is_the_distance() {
    // The height is measured along the ellipsoid normal, so 3 m up is 3 m away.
    let low = at(52.52, 13.40, 34.0);
    let high = at(52.52, 13.40, 37.0);
    let diff = (low.distance(&high) - 3.0).abs();
    assert!(diff < 1e-6, "diff = {diff} m");
}

#[test]
fn test_longitudes_360_degrees_apart_name_the_same_point() {
    let a = at(10.0, 190.0, 5.0);
    let b = at(10.0, -170.0, 5.0);
    assert!(a.distance(&b) < 1e-6);
}

#[test]
fn test_distance_is_symmetric_and_satisfies_the_triangle_inequality() {
    let p = at(52.520008, 13.404954, 34.0);
    let q = at(48.856613, 2.352222, 35.0);
    let r = at(-33.8688, 151.2093, 58.0);
    assert_eq!(p.distance(&q), q.distance(&p));
    assert!(p.distance(&r) <= p.distance(&q) + q.distance(&r));
    assert_eq!(p.distance(&p), 0.0);
}

#[test]
fn test_a_datum_other_than_wgs84_has_no_distance() {
    // Only an ellipsoidal height enters the geocentric conversion; a gravity-related, pressure or
    // terrain altitude does not, so the distance is undefined.
    let ellipsoidal = at(52.52, 13.40, 34.0);
    for datum in [
        VerticalDatum::EGM96,
        VerticalDatum::EGM2008,
        VerticalDatum::ISA,
        VerticalDatum::Terrain,
    ] {
        let other: GeoSpace<f64> = GeoSpace::new(2, 52.52, 13.40, 34.0, datum).unwrap();
        assert!(other.distance(&other).is_nan(), "{datum:?} with itself");
        assert!(
            ellipsoidal.distance(&other).is_nan(),
            "WGS84 with {datum:?}"
        );
        assert!(
            other.distance(&ellipsoidal).is_nan(),
            "{datum:?} with WGS84"
        );
    }
}

#[test]
fn test_the_iogp_worked_example_at_every_precision() {
    // Row K. The same fixture at f32 and Float106. f32 carries about 7 significant digits, so a
    // distance near 5.75e6 m resolves to a few metres; Float106 resolves below the millimetre the
    // fixture is published to.
    let lat = 53.0 + 48.0 / 60.0 + 33.820 / 3600.0;
    let lon = 2.0 + 7.0 / 60.0 + 46.380 / 3600.0;
    let (x, y, z) = (3_771_793.968 - A, 140_253.342, 5_124_304.349);
    let expected = (x * x + y * y + z * z).sqrt();

    let north_sea =
        GeoSpace::new(1, lat as f32, lon as f32, 73.0_f32, VerticalDatum::WGS84).unwrap();
    let origin = GeoSpace::new(2, 0.0_f32, 0.0, 0.0, VerticalDatum::WGS84).unwrap();
    let diff = (f64::from(north_sea.distance(&origin)) - expected).abs();
    assert!(diff < 4.0, "f32: diff = {diff} m");

    let wide = |v: f64| deep_causality_num::Float106::from(v);
    let north_sea =
        GeoSpace::new(1, wide(lat), wide(lon), wide(73.0), VerticalDatum::WGS84).unwrap();
    let origin = GeoSpace::new(2, wide(0.0), wide(0.0), wide(0.0), VerticalDatum::WGS84).unwrap();
    let diff = (f64::from(north_sea.distance(&origin)) - expected).abs();
    assert!(diff < 0.002, "Float106: diff = {diff} m");
}
