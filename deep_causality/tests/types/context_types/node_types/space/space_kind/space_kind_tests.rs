// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::*;
use std::f64::consts::FRAC_1_SQRT_2;

#[test]
fn test_identifiable_trait() {
    let sk = SpaceKind::Geo(GeoSpace::new(42, 10.0, 20.0, 30.0));
    assert_eq!(sk.id(), 42);
}

#[test]
fn test_coordinate_trait_geo() {
    let sk = SpaceKind::Geo(GeoSpace::new(1, 52.5, 13.4, 34.0));
    assert_eq!(sk.dimension(), 3);
    assert_eq!(*sk.coordinate(0).unwrap(), 52.5);
    assert_eq!(*sk.coordinate(1).unwrap(), 13.4);
    assert_eq!(*sk.coordinate(2).unwrap(), 34.0);
}

#[test]
fn test_coordinate_trait_quaternion() {
    let sk = SpaceKind::Quaternion(QuaternionSpace::new(5, 1.0, 0.0, 0.0, 0.0));
    assert_eq!(sk.dimension(), 4);
    assert_eq!(*sk.coordinate(0).unwrap(), 1.0);
    assert_eq!(*sk.coordinate(3).unwrap(), 0.0);
}

#[test]
fn test_coordinate_out_of_bounds() {
    let sk = SpaceKind::Euclidean(EuclideanSpace::new(2, 1.0, 2.0, 3.0));
    let res = sk.coordinate(3);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let sk = SpaceKind::Ned(NedSpace::new(1, 100.0, 50.0, 10.0));
    let output = format!("{}", sk);
    assert_eq!(output, "NedSpace(id=1, N=100.0000, E=50.0000, D=10.0000)");
}

#[test]
fn test_all_variants_id_and_display() {
    let geo = SpaceKind::Geo(GeoSpace::new(1, 10.0, 20.0, 30.0));
    let ecef = SpaceKind::Ecef(EcefSpace::new(2, 1.0, 2.0, 3.0));
    let eucl = SpaceKind::Euclidean(EuclideanSpace::new(3, 4.0, 5.0, 6.0));
    let ned = SpaceKind::Ned(NedSpace::new(4, 7.0, 8.0, 9.0));
    let quat = SpaceKind::Quaternion(QuaternionSpace::new(
        5,
        FRAC_1_SQRT_2,
        0.0,
        0.0,
        FRAC_1_SQRT_2,
    ));

    assert_eq!(geo.id(), 1);
    assert_eq!(ecef.id(), 2);
    assert_eq!(eucl.id(), 3);
    assert_eq!(ned.id(), 4);
    assert_eq!(quat.id(), 5);

    let _ = format!("{geo}");
    let _ = format!("{ecef}");
    let _ = format!("{eucl}");
    let _ = format!("{ned}");
    let _ = format!("{quat}");
}

#[test]
fn test_spatial_trait_is_implemented() {
    fn assert_spatial_impl<T: Spatial<f64>>() {}
    assert_spatial_impl::<SpaceKind>();
}
