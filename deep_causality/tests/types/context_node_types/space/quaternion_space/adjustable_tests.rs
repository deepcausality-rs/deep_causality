/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::*;

#[test]
fn test_update() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.5);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.5);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.5);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.5);

    let result = qspace.update(&grid);
    assert!(result.is_ok());

    assert_eq!(qspace.x(), 0.5);
    assert_eq!(qspace.y(), 0.5);
    assert_eq!(qspace.z(), 0.5);
    assert_eq!(qspace.w(), 0.5);
}

#[test]
fn test_adjust() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.1);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.1);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.1);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.1);

    let result = qspace.adjust(&grid);
    assert!(result.is_ok());

    assert_eq!(qspace.w(), 1.1);
    assert_eq!(qspace.x(), 0.1);
    assert_eq!(qspace.y(), 0.1);
    assert_eq!(qspace.z(), 0.1);
}

#[test]
fn test_update_w_fails_on_nan() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), f64::NAN); // w
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_w_fails_on_inf() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), f64::INFINITY); // w
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_w_fails_on_neg_inf() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), f64::NEG_INFINITY); // w
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_x_fails_on_nan() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), f64::NAN); // x
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_x_fails_on_inf() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), f64::INFINITY); // x
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_x_fails_on_neg_inf() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), f64::NEG_INFINITY); // x
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_y_fails_on_nan() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), f64::NAN); // Y
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_y_fails_on_inf() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), f64::INFINITY); // Y
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_y_fails_on_neg_inf() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), f64::NEG_INFINITY); // Y
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_z_fails_on_nan() {
    let mut qspace = QuaternionSpace::new(1, 0.1, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), f64::NAN); // Z

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_z_fails_on_inf() {
    let mut qspace = QuaternionSpace::new(1, 0.1, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), f64::INFINITY); // Z

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_z_fails_on_neg_inf() {
    let mut qspace = QuaternionSpace::new(1, 0.1, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), f64::NEG_INFINITY); // Z

    let result = qspace.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_w_fails_on_nan() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), f64::NAN); // w
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_x_fails_on_nan() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), f64::NAN); // x
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_y_fails_on_nan() {
    let mut qspace = QuaternionSpace::new(1, 1.0, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), f64::NAN); // Y
    grid.set(PointIndex::new4d(0, 0, 0, 3), 0.0);

    let result = qspace.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_z_fails_on_nan() {
    let mut qspace = QuaternionSpace::new(1, 0.1, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 1> = ArrayGrid::new(ArrayType::Array4D);

    grid.set(PointIndex::new4d(0, 0, 0, 0), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 1), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 2), 0.0);
    grid.set(PointIndex::new4d(0, 0, 0, 3), f64::NAN); // Z

    let result = qspace.adjust(&grid);
    assert!(result.is_err());
}
