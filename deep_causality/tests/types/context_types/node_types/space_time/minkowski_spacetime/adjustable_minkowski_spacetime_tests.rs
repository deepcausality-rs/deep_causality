/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::*;

#[test]
fn test_minkowski_spacetime_update_success() {
    let mut s = MinkowskiSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 10.0);
    grid.set(PointIndex::new3d(0, 0, 1), 20.0);
    grid.set(PointIndex::new3d(0, 0, 2), 30.0);
    grid.set(PointIndex::new3d(0, 0, 3), 40.0);

    let result = s.update(&grid);
    assert!(result.is_ok());
    assert_eq!(s.x(), 10.0);
    assert_eq!(s.y(), 20.0);
    assert_eq!(s.z(), 30.0);
    assert_eq!(*s.t(), 40.0);
}

#[test]
fn test_minkowski_spacetime_update_with_nan_should_fail() {
    let mut s = MinkowskiSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);
    grid.set(PointIndex::new3d(0, 0, 3), 1.0);

    let result = s.update(&grid);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not a finite value"));
}

#[test]
fn test_minkowski_spacetime_adjust_success() {
    let mut s = MinkowskiSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 0.1);
    grid.set(PointIndex::new3d(0, 0, 1), 0.2);
    grid.set(PointIndex::new3d(0, 0, 2), 0.3);
    grid.set(PointIndex::new3d(0, 0, 3), 0.4);

    let result = s.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(s.x(), 1.1);
    assert_eq!(s.y(), 2.2);
    assert_eq!(s.z(), 3.3);
    assert_eq!(*s.t(), 4.4);
}

#[test]
fn test_minkowski_spacetime_adjust_with_infinity_should_fail() {
    let mut s = MinkowskiSpacetime::new(1, f64::MAX, 0.0, 0.0, 0.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::INFINITY);
    grid.set(PointIndex::new3d(0, 0, 1), 0.0);
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);
    grid.set(PointIndex::new3d(0, 0, 3), 0.0);

    let result = s.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not a finite value"));
}

#[test]
fn test_minkowski_spacetime_update_fails_non_finite_x() {
    let mut s = MinkowskiSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN);
    grid.set(PointIndex::new3d(0, 0, 1), 2.0);
    grid.set(PointIndex::new3d(0, 0, 2), 3.0);
    grid.set(PointIndex::new3d(0, 0, 3), 4.0);
    assert!(s.update(&grid).is_err());
}

#[test]
fn test_minkowski_spacetime_update_fails_non_finite_y() {
    let mut s = MinkowskiSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY);
    grid.set(PointIndex::new3d(0, 0, 2), 3.0);
    grid.set(PointIndex::new3d(0, 0, 3), 4.0);
    assert!(s.update(&grid).is_err());
}

#[test]
fn test_minkowski_spacetime_update_fails_non_finite_z() {
    let mut s = MinkowskiSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 2.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::NEG_INFINITY);
    grid.set(PointIndex::new3d(0, 0, 3), 4.0);
    assert!(s.update(&grid).is_err());
}

#[test]
fn test_minkowski_spacetime_update_fails_non_finite_t() {
    let mut s = MinkowskiSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 2.0);
    grid.set(PointIndex::new3d(0, 0, 2), 3.0);
    grid.set(PointIndex::new3d(0, 0, 3), f64::NAN);
    assert!(s.update(&grid).is_err());
}
