// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::*;

#[test]
fn test_identifiable_trait() {
    let g = GeoSpace::new(1, 52.52, 13.40, 34.0);
    assert_eq!(g.id(), 1);
}

#[test]
fn test_coordinate_trait() {
    let g = GeoSpace::new(1, 52.52, 13.40, 34.0);

    assert_eq!(g.dimension(), 3);
    assert_eq!(*g.coordinate(0), 52.52);
    assert_eq!(*g.coordinate(1), 13.40);
    assert_eq!(*g.coordinate(2), 34.0);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_coordinate_out_of_bounds() {
    let g = GeoSpace::new(1, 0.0, 0.0, 0.0);
    let _ = g.coordinate(3); // should panic
}

#[test]
fn test_display_trait() {
    let g = GeoSpace::new(1, 52.520008, 13.404954, 34.0);
    let output = format!("{}", g);
    assert_eq!(output, "GeoSpace(id=1, x=52.5200, y=13.4050, z=34.0000)");
}

#[test]
fn test_metric_trait() {
    let a = GeoSpace::new(1, 0.0, 0.0, 0.0); // Equator, Prime Meridian
    let b = GeoSpace::new(2, 0.0, 3.0, 0.0); // 3° east, same latitude

    let distance = a.distance(&b);

    // Roughly ~333.6 km along the equator for 3° longitude
    let expected = 333_584.77995765815;

    let delta = 1e-2; // 1 cm tolerance

    let res = (distance - expected).abs();
    dbg!(res);
    dbg!(delta);
    dbg!(res < delta);

    assert!(
        res < delta,
        "Distance mismatch: expected ~{}, got {}, diff = {}",
        expected,
        distance,
        res
    );
}

#[test]
fn test_spatial_trait_is_implemented() {
    fn assert_spatial_impl<T: Spatial<f64>>() {}
    assert_spatial_impl::<GeoSpace>();
}
