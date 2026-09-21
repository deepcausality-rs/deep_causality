/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::FloatType;
use deep_causality_context::*;

#[test]
fn test_identifiable_trait() {
    let space = EcefSpace::new(123, 1.0, 2.0, 3.0);
    assert_eq!(space.id(), 123);
}

#[test]
fn test_coordinate_trait() {
    let space = EcefSpace::new(1, 10.0, 20.0, 30.0);

    assert_eq!(space.dimension(), 3);
    assert_eq!(*space.coordinate(0).unwrap(), 10.0);
    assert_eq!(*space.coordinate(1).unwrap(), 20.0);
    assert_eq!(*space.coordinate(2).unwrap(), 30.0);
}

#[test]
fn test_coordinate_out_of_bounds() {
    let space = EcefSpace::new(1, 0.0, 0.0, 0.0);
    let res = space.coordinate(3);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let space = EcefSpace::new(1, 12.34, 56.78, 90.12);
    let output = format!("{space}");
    assert!(output.contains("EcefSpace(id=1"));
    assert!(output.contains("x=12.34"));
    assert!(output.contains("y=56.78"));
    assert!(output.contains("z=90.12"));
}

#[test]
fn test_metric_trait() {
    let a = EcefSpace::new(1, 0.0, 0.0, 0.0);
    let b = EcefSpace::new(2, 3.0, 4.0, 0.0);

    let distance = a.distance(&b);
    assert_eq!(distance, 5.0);
}

#[test]
fn test_spatial_trait_is_implemented() {
    fn assert_spatial_impl<T: Spatial>() {}
    assert_spatial_impl::<EcefSpace<FloatType>>();
}

// =============================================================================
// distance() — every axis carries, and the metric axioms hold
// =============================================================================

/// Pythagorean quadruples: `(dx, dy, dz, r)` with `dx² + dy² + dz² = r²` in integers, so both
/// sides are exact in binary floating point.
///
/// ECEF is a geocentric Cartesian frame, so the straight-line distance is the ordinary 3D norm.
/// Keeping `dz` nonzero and distinct matters here more than elsewhere: `z` is the polar axis, and
/// dropping it would leave every equatorial pair correct.
const QUADRUPLES: [(FloatType, FloatType, FloatType, FloatType); 6] = [
    (3.0, 4.0, 0.0, 5.0),
    (3.0, 4.0, 12.0, 13.0),
    (1.0, 2.0, 2.0, 3.0),
    (2.0, 3.0, 6.0, 7.0),
    (1.0, 4.0, 8.0, 9.0),
    (2.0, 6.0, 9.0, 11.0),
];

#[test]
fn test_distance_sums_all_three_axes() {
    for (dx, dy, dz, expected) in QUADRUPLES {
        let a = EcefSpace::new(1, 0.0, 0.0, 0.0);
        let b = EcefSpace::new(2, dx, dy, dz);
        assert_eq!(a.distance(&b), expected, "for ({dx}, {dy}, {dz})");
    }
}

#[test]
fn test_distance_is_translation_invariant() {
    // Reading a coordinate instead of a coordinate difference agrees with the origin case and
    // fails here. Earth-scale offsets, because ECEF coordinates are metres from the geocentre.
    let offset = (4_517_590.0 as FloatType, 837_890.0, 4_546_802.0);

    for (dx, dy, dz, expected) in QUADRUPLES {
        let a = EcefSpace::new(1, offset.0, offset.1, offset.2);
        let b = EcefSpace::new(2, offset.0 + dx, offset.1 + dy, offset.2 + dz);
        assert_eq!(a.distance(&b), expected, "for ({dx}, {dy}, {dz})");
    }
}

#[test]
fn test_distance_is_symmetric_and_vanishes_only_on_coincidence() {
    for (dx, dy, dz, expected) in QUADRUPLES {
        let a = EcefSpace::new(1, 0.0, 0.0, 0.0);
        let b = EcefSpace::new(2, dx, dy, dz);

        assert_eq!(a.distance(&b), b.distance(&a), "for ({dx}, {dy}, {dz})");
        assert!(expected > 0.0);
        assert_eq!(a.distance(&a), 0.0);
        assert_eq!(b.distance(&b), 0.0);
    }
}

#[test]
fn test_distance_between_the_poles_is_twice_the_polar_radius() {
    // The two ECEF poles. The semi-minor axis is the published WGS84 value
    // b = 6_356_752.314245 m (NIMA TR8350.2, third edition, table 3.1). Their separation is 2b
    // along z alone, which an implementation that never reads z reports as zero.
    let b_axis: FloatType = 6_356_752.314245;
    let north = EcefSpace::new(1, 0.0, 0.0, b_axis);
    let south = EcefSpace::new(2, 0.0, 0.0, -b_axis);

    assert_eq!(north.distance(&south), 2.0 * b_axis);
}
