/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{Adjustable, DiscreteTime, Temporal, TimeScale};
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

#[test]
fn test_update() {
    let mut dt = DiscreteTime::new(1, TimeScale::Steps, 0);
    assert_eq!(dt.time_unit(), 0);

    let grid: ArrayGrid<u64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 42);

    let result = dt.update(&grid);
    assert!(result.is_ok());
    assert_eq!(dt.time_unit(), 42);
}

#[test]
fn test_adjust_success() {
    let mut dt = DiscreteTime::new(1, TimeScale::Steps, 10);
    let grid: ArrayGrid<u64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 5);

    let result = dt.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(dt.time_unit(), 15); // 10 + 5
}

#[test]
fn test_adjust_zero() {
    let mut dt = DiscreteTime::new(1, TimeScale::Steps, 0);
    let grid: ArrayGrid<u64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 0);

    let result = dt.adjust(&grid);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("new time is ZERO"));
}

#[test]
fn test_adjust_overflow() {
    let mut dt = DiscreteTime::new(1, TimeScale::Steps, u64::MAX);
    let grid: ArrayGrid<u64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 42);

    let result = dt.adjust(&grid);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Adjustment failed, u64 overflow"));
}
