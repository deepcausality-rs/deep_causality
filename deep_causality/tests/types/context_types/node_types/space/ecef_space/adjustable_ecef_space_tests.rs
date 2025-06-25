/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{ArrayGrid, ArrayType};
use deep_causality::prelude::{Adjustable, Coordinate, EcefSpace, Identifiable, Metric, Spatial};

#[test]
fn test_construction_and_accessors() {
    let space = EcefSpace::new(42, 1.0, 2.0, 3.0);
    assert_eq!(space.id(), 42);
    assert_eq!(space.x(), 1.0);
    assert_eq!(space.y(), 2.0);
    assert_eq!(space.z(), 3.0);
}

#[test]
fn test_coordinate_trait() {
    let space = EcefSpace::new(0, 1.0, 2.0, 3.0);
    assert_eq!(space.dimension(), 3);
    assert_eq!(*space.coordinate(0).unwrap(), 1.0);
    assert_eq!(*space.coordinate(1).unwrap(), 2.0);
    assert_eq!(*space.coordinate(2).unwrap(), 3.0);
}

#[test]
fn test_coordinate_trait_out_of_bounds() {
    let space = EcefSpace::new(0, 1.0, 2.0, 3.0);
    let res = space.coordinate(3);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let space = EcefSpace::new(1, 1.2345, 2.3456, 3.4567);

    // dbg!(&space);
    let output = format!("{}", space);
    dbg!(&output);
    assert!(output.contains("(id=1"));
    assert!(output.contains("x=1.2345"));
    assert!(output.contains("y=2.3456"));
    assert!(output.contains("z=3.4567"));
}

#[test]
fn test_partial_eq_and_clone() {
    let a = EcefSpace::new(10, 1.0, 2.0, 3.0);
    let b = a.clone();
    let c = EcefSpace::new(10, 1.0, 2.0, 3.0);
    let d = EcefSpace::new(11, 1.0, 2.0, 3.0);

    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_ne!(a, d);
}

#[test]
fn test_metric_trait() {
    let a = EcefSpace::new(0, 0.0, 0.0, 0.0);
    let b = EcefSpace::new(1, 3.0, 4.0, 0.0);
    assert_eq!(a.distance(&b), 5.0); // 3-4-5 triangle
}

#[test]
fn test_ecef_space_trait_default_impls() {
    let mut space = EcefSpace::new(1, 1.0, 2.0, 3.0);
    let dummy_grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);

    let update_result = space.update(&dummy_grid);
    let adjust_result = space.adjust(&dummy_grid);

    assert!(update_result.is_ok());
    assert!(adjust_result.is_ok());
}

#[test]
fn test_spatial_trait_marker() {
    fn assert_spatial<T: Spatial<f64>>() {}
    assert_spatial::<EcefSpace>();
}
