/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

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
    assert_eq!(*g.coordinate(0).unwrap(), 52.52);
    assert_eq!(*g.coordinate(1).unwrap(), 13.40);
    assert_eq!(*g.coordinate(2).unwrap(), 34.0);
}

#[test]
fn test_coordinate_out_of_bounds() {
    let g = GeoSpace::new(1, 0.0, 0.0, 0.0);
    let res = g.coordinate(3);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let g = GeoSpace::new(1, 52.520008, 13.404954, 34.0);
    let output = format!("{g}");
    assert_eq!(
        output,
        "GeoSpace(id=1, lat=52.5200, lon=13.4050, alt=34.0000)"
    );
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
        "Distance mismatch: expected ~{expected}, got {distance}, diff = {res}"
    );
}

#[test]
fn test_spatial_trait_is_implemented() {
    fn assert_spatial_impl<T: Spatial<f64>>() {}
    assert_spatial_impl::<GeoSpace>();
}
