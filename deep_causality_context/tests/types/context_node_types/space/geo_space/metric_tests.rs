/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::FloatType;
use deep_causality_context::*;

/// Earth's mean radius in meters, as used by `GeoSpace::distance`.
const EARTH_RADIUS: FloatType = 6_371_000.0;

/// Half the great circle, which is the surface distance between antipodal points.
fn half_circumference() -> FloatType {
    core::f64::consts::PI * EARTH_RADIUS
}

#[test]
fn test_distance_is_finite_at_antipodal_points() {
    // `a = sin²(Δφ/2) + cosφ₁ cosφ₂ sin²(Δλ/2)` is at most 1 in exact arithmetic, and this pair
    // rounds it to 1.0000000000000002. Without the clamp, `(1 - a).sqrt()` is NaN and the whole
    // distance is NaN.
    let a = GeoSpace::new(1, -44.9953, 0.0, 0.0, VerticalDatum::WGS84);
    let b = GeoSpace::new(2, 44.9953, 180.0, 0.0, VerticalDatum::WGS84);

    let distance: FloatType = a.distance(&b);

    assert!(distance.is_finite(), "distance is not finite: {distance}");

    let expected = half_circumference();
    let diff = (distance - expected).abs();
    assert!(
        diff < 1.0,
        "expected ~{expected} m, got {distance} m, diff = {diff} m"
    );
}

#[test]
fn test_distance_is_finite_across_an_antipodal_sweep() {
    // One pair proves the clamp fires; the sweep shows no latitude on the antipodal meridian
    // pushes `a` past the clamp into NaN again.
    let mut i = 0;
    while i <= 900_000 {
        let lat = i as FloatType / 10_000.0 - 45.0;
        let a = GeoSpace::new(1, lat, 0.0, 0.0, VerticalDatum::WGS84);
        let b = GeoSpace::new(2, -lat, 180.0, 0.0, VerticalDatum::WGS84);

        let distance = a.distance(&b);
        assert!(
            distance.is_finite(),
            "distance is not finite at lat {lat}: {distance}"
        );

        i += 1;
    }
}

#[test]
fn test_distance_with_equal_datums_includes_the_altitude_term() {
    // Same point horizontally, 3 m apart vertically against the same reference: the altitude
    // difference is the whole distance.
    let a = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::EGM96);
    let b = GeoSpace::new(2, 52.52, 13.40, 37.0, VerticalDatum::EGM96);

    let distance: FloatType = a.distance(&b);
    let diff = (distance - 3.0).abs();
    assert!(diff < 1e-9, "expected 3 m, diff = {diff}");
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "requires both operands to use the same VerticalDatum")]
fn test_distance_across_mismatched_datums_panics_in_debug() {
    // Documented precondition: an ellipsoidal height and an orthometric height are not
    // subtractable, so the altitude term of the distance is undefined here.
    let ellipsoidal = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84);
    let orthometric = GeoSpace::new(2, 52.52, 13.40, 37.0, VerticalDatum::EGM2008);

    let _ = ellipsoidal.distance(&orthometric);
}

#[test]
#[cfg(not(debug_assertions))]
fn test_distance_across_mismatched_datums_is_computed_as_if_equal_in_release() {
    // The same call in release drops the check and subtracts the two numbers anyway, which is
    // exactly why the precondition is documented rather than merely assumed.
    let ellipsoidal = GeoSpace::new(1, 52.52, 13.40, 34.0, VerticalDatum::WGS84);
    let orthometric = GeoSpace::new(2, 52.52, 13.40, 37.0, VerticalDatum::EGM2008);

    let distance: FloatType = ellipsoidal.distance(&orthometric);
    let diff = (distance - 3.0).abs();
    assert!(diff < 1e-9, "expected 3 m, diff = {diff}");
}
