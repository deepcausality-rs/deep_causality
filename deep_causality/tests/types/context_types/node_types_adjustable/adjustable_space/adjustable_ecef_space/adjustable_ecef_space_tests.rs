// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::{ArrayGrid, ArrayType};
use deep_causality::prelude::*;

#[test]
fn test_construction_and_accessors() {
    let space = AdjustableEcefSpace::new(42, 1.0, 2.0, 3.0);
    assert_eq!(space.id(), 42);
    assert_eq!(space.x(), 1.0);
    assert_eq!(space.y(), 2.0);
    assert_eq!(space.z(), 3.0);
}

#[test]
fn test_coordinate_trait() {
    let space = AdjustableEcefSpace::new(0, 1.0, 2.0, 3.0);
    assert_eq!(space.dimension(), 3);
    assert_eq!(*space.coordinate(0), 1.0);
    assert_eq!(*space.coordinate(1), 2.0);
    assert_eq!(*space.coordinate(2), 3.0);
}

#[test]
#[should_panic(expected = "AdjustableEcefSpace: coordinate index out of bounds")]
fn test_coordinate_trait_out_of_bounds() {
    let space = AdjustableEcefSpace::new(0, 1.0, 2.0, 3.0);
    let _ = space.coordinate(3);
}

#[test]
fn test_display_trait() {
    let space = AdjustableEcefSpace::new(1, 1.2345, 2.3456, 3.4567);

    // dbg!(&space);
    let output = format!("{}", space);
    dbg!(&output);
    assert!(output.contains("EcefSpace(id=1"));
    assert!(output.contains("x=1.234"));
    assert!(output.contains("y=2.346"));
    assert!(output.contains("z=3.457"));
}

#[test]
fn test_partial_eq_and_clone() {
    let a = AdjustableEcefSpace::new(10, 1.0, 2.0, 3.0);
    let b = a.clone();
    let c = AdjustableEcefSpace::new(10, 1.0, 2.0, 3.0);
    let d = AdjustableEcefSpace::new(11, 1.0, 2.0, 3.0);

    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_ne!(a, d);
}

#[test]
fn test_metric_trait() {
    let a = AdjustableEcefSpace::new(0, 0.0, 0.0, 0.0);
    let b = AdjustableEcefSpace::new(1, 3.0, 4.0, 0.0);
    assert_eq!(a.distance(&b), 5.0); // 3-4-5 triangle
}

#[test]
fn test_adjustable_trait_default_impls() {
    let mut space = AdjustableEcefSpace::new(1, 1.0, 2.0, 3.0);
    let dummy_grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);

    let update_result = space.update(&dummy_grid);
    let adjust_result = space.adjust(&dummy_grid);

    assert!(update_result.is_ok());
    assert!(adjust_result.is_ok());
}

#[test]
fn test_spatial_trait_marker() {
    fn assert_spatial<T: Spatial<f64>>() {}
    assert_spatial::<AdjustableEcefSpace>();
}
