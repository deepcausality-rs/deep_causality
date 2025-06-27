/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

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
    fn assert_spatial_impl<T: Spatial<f64>>() {}
    assert_spatial_impl::<EcefSpace>();
}
