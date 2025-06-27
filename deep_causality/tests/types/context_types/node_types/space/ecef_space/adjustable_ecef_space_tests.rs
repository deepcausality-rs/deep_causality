/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::{Adjustable, Coordinate, EcefSpace, Identifiable, Metric, Spatial};

#[test]
fn test_ecef_space_adjustable_update_and_adjust() {
    let mut space = EcefSpace::new(1, 1.0, 2.0, 3.0);

    // Test update
    let mut update_grid: ArrayGrid<f64, 3, 3, 3, 3> = ArrayGrid::new(ArrayType::Array3D);
    update_grid.set(PointIndex::new3d(0, 0, 0), 10.0);
    update_grid.set(PointIndex::new3d(0, 0, 1), 20.0);
    update_grid.set(PointIndex::new3d(0, 0, 2), 30.0);
    let update_result = space.update(&update_grid);
    assert!(update_result.is_ok());
    assert_eq!(space.x(), 10.0);
    assert_eq!(space.y(), 20.0);
    assert_eq!(space.z(), 30.0);

    // Test adjust
    let mut adjust_grid: ArrayGrid<f64, 3, 3, 3, 3> = ArrayGrid::new(ArrayType::Array3D);
    adjust_grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    adjust_grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    adjust_grid.set(PointIndex::new3d(0, 0, 2), 1.0);
    let adjust_result = space.adjust(&adjust_grid);
    assert!(adjust_result.is_ok());
    assert_eq!(space.x(), 11.0);
    assert_eq!(space.y(), 21.0);
    assert_eq!(space.z(), 31.0);
}

#[test]
fn test_ecef_update_x_not_finite() {
    let mut space = EcefSpace::new(1, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 3> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    let result = space.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ecef_update_y_not_finite() {
    let mut space = EcefSpace::new(2, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 3> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    let result = space.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ecef_update_z_not_finite() {
    let mut space = EcefSpace::new(3, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 3> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::NEG_INFINITY);

    let result = space.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ecef_adjust_x_not_finite() {
    let mut space = EcefSpace::new(1, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 3> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    let result = space.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ecef_adjust_y_not_finite() {
    let mut space = EcefSpace::new(2, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 3> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);

    let result = space.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ecef_adjust_z_not_finite() {
    let mut space = EcefSpace::new(3, 1.0, 1.0, 1.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 3> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::NEG_INFINITY);

    let result = space.adjust(&grid);
    assert!(result.is_err());
}
