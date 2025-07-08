/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::*;

#[test]
fn test_update_success() {
    let mut s = EuclideanSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

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
    assert_eq!(s.time_unit(), 40.0);
}

#[test]
fn test_update_nan_should_fail() {
    let mut s = EuclideanSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);
    grid.set(PointIndex::new3d(0, 0, 3), 1.0);

    let result = s.update(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not a finite value"));
}

#[test]
fn test_adjust_success() {
    let mut s = EuclideanSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);
    grid.set(PointIndex::new3d(0, 0, 3), 1.0);

    let result = s.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(s.x(), 2.0);
    assert_eq!(s.y(), 3.0);
    assert_eq!(s.z(), 4.0);
    assert_eq!(s.time_unit(), 5.0);
}

#[test]
fn test_update_fails_with_non_finite_x() {
    let mut s = EuclideanSpacetime::new(1, f64::MAX, 1.0, 1.0, 1.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::INFINITY);
    grid.set(PointIndex::new3d(0, 0, 1), 0.0);
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);
    grid.set(PointIndex::new3d(0, 0, 3), 0.0);

    let result = s.update(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not a finite value"));
}

#[test]
fn test_update_fails_with_non_finite_y() {
    let mut s = EuclideanSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0); // x
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN); // y (invalid)
    grid.set(PointIndex::new3d(0, 0, 2), 3.0); // z
    grid.set(PointIndex::new3d(0, 0, 3), 4.0); // t
    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_fails_with_non_finite_z() {
    let mut s = EuclideanSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 2.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::INFINITY); // z (invalid)
    grid.set(PointIndex::new3d(0, 0, 3), 4.0);
    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_fails_with_non_finite_t() {
    let mut s = EuclideanSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 2.0);
    grid.set(PointIndex::new3d(0, 0, 2), 3.0);
    grid.set(PointIndex::new3d(0, 0, 3), f64::NEG_INFINITY); // t (invalid)
    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_fails_with_non_finite_x() {
    let mut s = EuclideanSpacetime::new(1, f64::MAX, 1.0, 1.0, 1.0, TimeScale::Second);

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
fn test_adjust_fails_with_non_finite_y() {
    let mut s = EuclideanSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0); // x
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN); // y (invalid)
    grid.set(PointIndex::new3d(0, 0, 2), 3.0); // z
    grid.set(PointIndex::new3d(0, 0, 3), 4.0); // t
    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_fails_with_non_finite_z() {
    let mut s = EuclideanSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 2.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::INFINITY); // z (invalid)
    grid.set(PointIndex::new3d(0, 0, 3), 4.0);
    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_fails_with_non_finite_t() {
    let mut s = EuclideanSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 2.0);
    grid.set(PointIndex::new3d(0, 0, 2), 3.0);
    grid.set(PointIndex::new3d(0, 0, 3), f64::NEG_INFINITY); // t (invalid)
    let result = s.adjust(&grid);
    assert!(result.is_err());
}
