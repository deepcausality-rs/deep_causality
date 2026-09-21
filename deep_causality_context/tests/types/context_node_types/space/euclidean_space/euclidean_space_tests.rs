/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::FloatType;
use deep_causality_context::*;

#[test]
fn test_identifiable_trait() {
    let space = EuclideanSpace::new(42, 1.0, 2.0, 3.0);
    assert_eq!(space.id(), 42);
}

#[test]
fn test_coordinate_trait() {
    let space = EuclideanSpace::new(1, 10.0, 20.0, 30.0);

    assert_eq!(space.dimension(), 3);
    assert_eq!(*space.coordinate(0).unwrap(), 10.0);
    assert_eq!(*space.coordinate(1).unwrap(), 20.0);
    assert_eq!(*space.coordinate(2).unwrap(), 30.0);
}

#[test]
fn test_coordinate_out_of_bounds() {
    let space = EuclideanSpace::new(1, 0.0, 0.0, 0.0);
    let res = space.coordinate(3);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let space = EuclideanSpace::new(1, 3.00, 1.59, 2.65);
    let output = format!("{space}");
    assert!(output.contains("EuclideanSpace(id=1"));
    assert!(output.contains("x=3.00"));
    assert!(output.contains("y=1.59"));
    assert!(output.contains("z=2.65"));
}

#[test]
fn test_metric_trait() {
    let a = EuclideanSpace::new(1, 0.0, 0.0, 0.0);
    let b = EuclideanSpace::new(2, 3.0, 4.0, 0.0);

    let distance = a.distance(&b);
    assert_eq!(distance, 5.0);
}

#[test]
fn test_spatial_trait_is_implemented() {
    fn assert_spatial_impl<T: Spatial>() {}
    assert_spatial_impl::<EuclideanSpace<FloatType>>();
}

// =============================================================================
// distance() — every axis carries, and the metric axioms hold
// =============================================================================

/// Pythagorean quadruples: `(dx, dy, dz, r)` with `dx² + dy² + dz² = r²` in integers, so both
/// sides are exact in binary floating point and `assert_eq!` is the right comparison.
///
/// The first row degenerates in `z` and is kept only as the plane case. The rest keep all three
/// axes nonzero and distinct, so a dropped `dz*dz` shows up: on `(3, 4, 12)` it reports 5 where
/// the answer is 13.
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
        let a = EuclideanSpace::new(1, 0.0, 0.0, 0.0);
        let b = EuclideanSpace::new(2, dx, dy, dz);
        assert_eq!(a.distance(&b), expected, "for ({dx}, {dy}, {dz})");
    }
}

#[test]
fn test_distance_is_translation_invariant() {
    // Reading a coordinate instead of a coordinate difference agrees with the origin case and
    // fails here, because the same separation is placed away from the origin.
    let offset = (7.0 as FloatType, -11.0, 23.0);

    for (dx, dy, dz, expected) in QUADRUPLES {
        let a = EuclideanSpace::new(1, offset.0, offset.1, offset.2);
        let b = EuclideanSpace::new(2, offset.0 + dx, offset.1 + dy, offset.2 + dz);
        assert_eq!(a.distance(&b), expected, "for ({dx}, {dy}, {dz})");
    }
}

#[test]
fn test_distance_is_symmetric_and_vanishes_only_on_coincidence() {
    for (dx, dy, dz, expected) in QUADRUPLES {
        let a = EuclideanSpace::new(1, 0.0, 0.0, 0.0);
        let b = EuclideanSpace::new(2, dx, dy, dz);

        assert_eq!(a.distance(&b), b.distance(&a), "for ({dx}, {dy}, {dz})");
        assert!(expected > 0.0);
        assert_eq!(a.distance(&a), 0.0);
        assert_eq!(b.distance(&b), 0.0);
    }
}

#[test]
fn test_distance_obeys_the_triangle_inequality() {
    // The metric axiom, with a point off the line so the inequality is strict.
    let a = EuclideanSpace::new(1, 0.0, 0.0, 0.0);
    let b = EuclideanSpace::new(2, 3.0, 4.0, 12.0);
    let c = EuclideanSpace::new(3, -5.0, 8.0, 1.0);

    assert!(a.distance(&b) <= a.distance(&c) + c.distance(&b));
    assert!(a.distance(&b) < a.distance(&c) + c.distance(&b));
}
