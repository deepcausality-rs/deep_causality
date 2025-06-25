// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::*;

#[test]
fn test_identifiable_trait() {
    let n = NedSpace::new(42, 1.0, 2.0, 3.0);
    assert_eq!(n.id(), 42);
}

#[test]
fn test_coordinate_trait() {
    let n = NedSpace::new(1, 10.0, 20.0, 30.0);

    assert_eq!(n.dimension(), 3);
    assert_eq!(*n.coordinate(0).unwrap(), 10.0);
    assert_eq!(*n.coordinate(1).unwrap(), 20.0);
    assert_eq!(*n.coordinate(2).unwrap(), 30.0);
}

#[test]
fn test_coordinate_out_of_bounds() {
    let n = NedSpace::new(1, 0.0, 0.0, 0.0);
    let res = n.coordinate(3);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let n = NedSpace::new(1, 100.0, 50.0, 10.0);
    let output = format!("{}", n);
    assert_eq!(output, "NedSpace(id=1, N=100.0000, E=50.0000, D=10.0000)");
}

#[test]
fn test_metric_trait() {
    let n1 = NedSpace::new(1, 0.0, 0.0, 0.0);
    let n2 = NedSpace::new(2, 100.0, 50.0, 10.0);

    let dist = n1.distance(&n2);
    let expected = (100.0f64.powi(2) + 50.0f64.powi(2) + 10.0f64.powi(2)).sqrt();

    assert_eq!(dist, expected);
}

#[test]
fn test_spatial_trait_is_implemented() {
    fn assert_spatial_impl<T: Spatial<f64>>() {}
    assert_spatial_impl::<NedSpace>();
}
