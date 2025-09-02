/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{Adjustable, TangentSpacetime, Temporal};
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

#[test]
fn test_tangent_spacetime_update_success() {
    let mut t = TangentSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);
    grid.set(PointIndex::new3d(0, 0, 1), 2.0);
    grid.set(PointIndex::new3d(0, 0, 2), 3.0);
    grid.set(PointIndex::new3d(0, 0, 3), 4.0);

    let result = t.update(&grid);
    assert!(result.is_ok());
    assert_eq!(t.x(), 1.0);
    assert_eq!(t.y(), 2.0);
    assert_eq!(t.z(), 3.0);
    assert_eq!(t.time_unit(), 4.0);
}

#[test]
fn test_tangent_spacetime_update_invalid_x_inf() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::INFINITY);

    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_update_invalid_x_nan() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN);

    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_update_invalid_y_inf() {
    let mut t = TangentSpacetime::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY);

    let result = t.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_update_invalid_y_nan() {
    let mut t = TangentSpacetime::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN);

    let result = t.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_update_invalid_z_inf() {
    let mut t = TangentSpacetime::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 2), f64::INFINITY);

    let result = t.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_update_invalid_z_nan() {
    let mut t = TangentSpacetime::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 2), f64::NAN);

    let result = t.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_update_invalid_t_inf() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 3), f64::INFINITY);

    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_update_invalid_t_neg_inf() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 3), f64::NEG_INFINITY);

    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_update_invalid_t_nan() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 3), f64::NAN);

    let result = s.update(&grid);
    assert!(result.is_err());
}
