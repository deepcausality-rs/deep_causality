/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::*;

#[test]
fn test_ned_space_update() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 100.0); // north
    grid.set(PointIndex::new3d(0, 0, 1), 50.0); // east
    grid.set(PointIndex::new3d(0, 0, 2), 10.0); // down

    let result = ned.update(&grid);
    assert!(result.is_ok());

    assert_eq!(ned.north(), 100.0);
    assert_eq!(ned.east(), 50.0);
    assert_eq!(ned.down(), 10.0);
}

#[test]
fn test_ned_space_adjust() {
    let mut ned = NedSpace::new(1, 100.0, 50.0, 10.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 25.0); // north adjustment
    grid.set(PointIndex::new3d(0, 0, 1), -10.0); // east adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 5.0); // down adjustment

    let result = ned.adjust(&grid);
    assert!(result.is_ok());

    assert_eq!(ned.north(), 125.0);
    assert_eq!(ned.east(), 40.0);
    assert_eq!(ned.down(), 15.0);
}

#[test]
fn test_ned_space_adjust_north_fails_on_nan() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN); // North: Invalid adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0);
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);

    let result = ned.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ned_space_adjust_north_fails_on_inf() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::INFINITY); // North: Invalid adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0);
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);

    let result = ned.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ned_space_adjust_east_fails_on_nan() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0);
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN); // East: Invalid adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);

    let result = ned.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ned_space_adjust_east_fails_on_inf() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY); // East: Invalid adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);

    let result = ned.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ned_space_adjust_down_fails_on_nan() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0);
    grid.set(PointIndex::new3d(0, 0, 1), 0.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::NAN); // Down: Invalid adjustment

    let result = ned.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ned_space_adjust_down_fails_on_inf() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0);
    grid.set(PointIndex::new3d(0, 0, 1), 0.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::INFINITY); // Down: Invalid adjustment

    let result = ned.adjust(&grid);
    assert!(result.is_err());
}
