// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::*;

#[test]
fn test_quaternion_space_update() {
    let mut qspace = AdjustableQuaternionSpace::new(1, [1.0, 0.0, 0.0, 0.0]);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.5);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.5);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.5);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.5);

    let result = qspace.update(&grid);
    assert!(result.is_ok());
    assert_eq!(qspace.quat(), [0.5, 0.5, 0.5, 0.5]);
}

#[test]
fn test_quaternion_space_adjust() {
    let mut qspace = AdjustableQuaternionSpace::new(1, [0.1, 0.1, 0.1, 0.1]);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.1);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.1);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.1);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.1);

    let result = qspace.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(qspace.quat(), [0.2, 0.2, 0.2, 0.2]);
}

#[test]
fn test_quaternion_space_adjust_fails_on_nonfinite() {
    let mut qspace = AdjustableQuaternionSpace::new(1, [0.1, 0.1, 0.1, 0.1]);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), f64::INFINITY);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_quaternion_space_update_fails_on_nan() {
    let mut qspace = AdjustableQuaternionSpace::new(1, [1.0, 0.0, 0.0, 0.0]);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), f64::NAN);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}
