// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::*;

#[test]
fn test_identifiable_trait() {
    let space = EuclideanSpace::new(42, 1.0, 2.0, 3.0);
    assert_eq!(space.id(), 42);
}

#[test]
fn test_coordinate_trait() {
    let space = EuclideanSpace::new(1, 10.0, 20.0, 30.0);

    assert_eq!(space.dimension(), 3);
    assert_eq!(*space.coordinate(0), 10.0);
    assert_eq!(*space.coordinate(1), 20.0);
    assert_eq!(*space.coordinate(2), 30.0);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_coordinate_out_of_bounds() {
    let space = EuclideanSpace::new(1, 0.0, 0.0, 0.0);
    let _ = space.coordinate(3); // should panic
}

#[test]
fn test_display_trait() {
    let space = EuclideanSpace::new(1, 3.14, 1.59, 2.65);
    let output = format!("{}", space);
    assert_eq!(output, "EuclideanSpace(id=1, x=3.1400, y=1.5900, z=2.6500)");
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
    fn assert_spatial_impl<T: Spatial<f64>>() {}
    assert_spatial_impl::<EuclideanSpace>();
}
