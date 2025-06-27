/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

#[test]
fn test_identifiable_trait() {
    let q = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(q.id(), 1);
}

#[test]
fn test_coordinate_trait() {
    let q = QuaternionSpace::new(2, 0.5, 0.5, 0.5, 0.5);

    assert_eq!(q.dimension(), 4);
    assert_eq!(*q.coordinate(0).unwrap(), 0.5);
    assert_eq!(*q.coordinate(1).unwrap(), 0.5);
    assert_eq!(*q.coordinate(2).unwrap(), 0.5);
    assert_eq!(*q.coordinate(3).unwrap(), 0.5);
}

#[test]
fn test_coordinate_out_of_bounds() {
    let q = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let res = q.coordinate(4);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let q = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let output = format!("{q}");
    assert!(output.contains("QuaternionSpace(id=1"));
    assert!(output.contains("w=1.0000"));
    assert!(output.contains("x=0.0000"));
    assert!(output.contains("y=0.0000"));
    assert!(output.contains("z=0.0000"));
}

#[test]
fn test_metric_trait() {
    let q1 = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let q2 = QuaternionSpace::new(2, 0.0, 1.0, 0.0, 0.0);

    let dist = q1.distance(&q2);
    assert_eq!(dist, (2.0f64).sqrt()); // sqrt((1 - 0)^2 + (0 - 1)^2)
}

#[test]
fn test_spatial_trait_is_implemented() {
    fn assert_spatial_impl<T: Spatial<f64>>() {}
    assert_spatial_impl::<QuaternionSpace>();
}
