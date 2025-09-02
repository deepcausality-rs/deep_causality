/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{Adjustable, EntropicTime, Temporal};
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

#[test]
fn test_update() {
    let mut t = EntropicTime::new(1, 0);
    assert_eq!(t.time_unit(), 0);

    let grid: ArrayGrid<u64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 123);

    let result = t.update(&grid);
    assert!(result.is_ok());
    assert_eq!(t.time_unit(), 123);
}

#[test]
fn test_adjust_valid() {
    let mut t = EntropicTime::new(1, 10);
    let grid: ArrayGrid<u64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 5);

    let result = t.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(t.time_unit(), 15);
}

#[test]
fn test_adjust_zero() {
    let mut t = EntropicTime::new(1, 42);
    let grid: ArrayGrid<u64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 0);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("new time is ZERO"));
}

#[test]
fn test_adjust_overflow() {
    let mut t = EntropicTime::new(1, u64::MAX);
    let grid: ArrayGrid<u64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 1);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("u64 overflow"));
}
