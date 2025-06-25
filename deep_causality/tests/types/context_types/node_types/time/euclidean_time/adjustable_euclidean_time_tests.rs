/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::{Adjustable, EuclideanTime, Temporal, TimeScale};

#[test]
fn test_euclidean_time_update_success() {
    let mut t = EuclideanTime::new(1, TimeScale::Second, 0.0);
    assert_eq!(t.time_unit(), 0.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 3.00);

    let result = t.update(&grid);
    assert!(result.is_ok());
    assert_eq!(t.time_unit(), 3.00);
}

#[test]
fn test_euclidean_time_adjust_success() {
    let mut t = EuclideanTime::new(1, TimeScale::Second, 1.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 2.0);

    let result = t.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(t.time_unit(), 3.0);
}

#[test]
fn test_euclidean_time_adjust_nan_input() {
    let mut t = EuclideanTime::new(1, TimeScale::Second, 1.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), f64::NAN);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("time is NaN"));
}

#[test]
fn test_euclidean_time_adjust_negative_input() {
    let mut t = EuclideanTime::new(1, TimeScale::Second, 5.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), -1.0);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("new time is NEGATIVE"));
}

#[test]
fn test_euclidean_time_adjust_result_is_nan() {
    let mut t = EuclideanTime::new(1, TimeScale::Second, f64::MAX);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), f64::INFINITY);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not finite"));
}

#[test]
fn test_euclidean_time_adjust_result_zero() {
    let mut t = EuclideanTime::new(1, TimeScale::Second, 0.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 0.0);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("new time is ZERO"));
}
