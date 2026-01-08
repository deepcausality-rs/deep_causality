/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

#[test]
fn test_update() {
    let mut t = LorentzianTime::new(1, TimeScale::Second, 0.0);
    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 42.0);

    let result = t.update(&grid);
    assert!(result.is_ok());
    assert_eq!(t.time_unit(), 42.0);
}

#[test]
fn test_adjust_success() {
    let mut t = LorentzianTime::new(1, TimeScale::Second, 1.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 3.0);

    let result = t.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(t.time_unit(), 4.0);
}

#[test]
fn test_adjust_negative_res() {
    let mut t = LorentzianTime::new(1, TimeScale::Second, 10.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), -12.0);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Adjustment failed, result is a negative number"));
}

#[test]
fn test_adjust_zero_result() {
    let mut t = LorentzianTime::new(1, TimeScale::Second, 0.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 0.0);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Adjustment failed, new time is ZERO"));
}

#[test]
fn test_adjust_nan_input() {
    let mut t = LorentzianTime::new(1, TimeScale::Second, 1.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), f64::NAN);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("NaN"));
}

#[test]
fn test_adjust_result_is_nan() {
    let mut t = LorentzianTime::new(1, TimeScale::Second, f64::MAX);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), f64::INFINITY);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not finite"));
}
