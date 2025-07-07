/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::*;

#[test]
fn test_display_trait() {
    let space = EuclideanSpace::new(5, 1.234, 5.678, 9.876);
    let output = format!("{space}");
    assert!(output.contains("EuclideanSpace(id=5"));
    assert!(output.contains("x=1.234"));
    assert!(output.contains("y=5.678"));
    assert!(output.contains("z=9.876"));
}

#[test]
fn test_adjustable_trait_update_and_adjust() {
    let mut space = EuclideanSpace::new(1, 1.0, 2.0, 3.0);

    // Use matching layout from your successful test suite
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 10.0); // x
    grid.set(PointIndex::new3d(0, 0, 1), 20.0); // y
    grid.set(PointIndex::new3d(0, 0, 2), 30.0); // z

    let update_result = space.update(&grid);
    assert!(update_result.is_ok());
    assert_eq!(space.x(), 10.0);
    assert_eq!(space.y(), 20.0);
    assert_eq!(space.z(), 30.0);

    let adjust_result = space.adjust(&grid);
    assert!(adjust_result.is_ok());
    assert_eq!(space.x(), 20.0);
    assert_eq!(space.y(), 40.0);
    assert_eq!(space.z(), 60.0);
}

#[test]
fn test_update_x_not_finite() {
    let mut space = EuclideanSpace::new(1, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    let result = space.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_y_not_finite() {
    let mut space = EuclideanSpace::new(2, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    let result = space.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_z_not_finite() {
    let mut space = EuclideanSpace::new(3, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::NEG_INFINITY);

    let result = space.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_x_not_finite() {
    let mut space = EuclideanSpace::new(1, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    let result = space.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_y_not_finite() {
    let mut space = EuclideanSpace::new(2, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    let result = space.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_z_not_finite() {
    let mut space = EuclideanSpace::new(3, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::NEG_INFINITY);

    let result = space.adjust(&grid);
    assert!(result.is_err());
}
