/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_context::*;
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

#[test]
fn test_update() {
    let mut t = MinkowskiTime::new(1, TimeScale::Second, 0.0);
    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 42.0);

    let result = t.update(&grid);
    assert!(result.is_ok());
    assert_eq!(t.time_unit(), 42.0);
}

#[test]
fn test_adjust_success() {
    let mut t = MinkowskiTime::new(1, TimeScale::Second, 1.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 3.0);

    let result = t.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(t.time_unit(), 4.0);
}

#[test]
fn test_update_refuses_a_value_that_is_not_finite() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut t = MinkowskiTime::new(1, TimeScale::Second, 5.0);
        let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
        grid.set(PointIndex::new1d(0), value);

        let err = t.update(&grid).unwrap_err().to_string();
        assert!(err.contains("not a finite value"), "{value}: {err}");
        assert_eq!(t.time_unit(), 5.0, "{value}: the time must be kept");
    }
}

#[test]
fn test_update_accepts_a_negative_and_a_zero_time() {
    for value in [-7.25, 0.0] {
        let mut t = MinkowskiTime::new(1, TimeScale::Second, 5.0);
        let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
        grid.set(PointIndex::new1d(0), value);

        assert!(t.update(&grid).is_ok(), "{value}");
        assert_eq!(t.time_unit(), value);
    }
}

/// `(start, shift, start + shift)`, exact in binary, covering every sign of start and result.
const TRANSLATIONS: [(f64, f64, f64); 5] = [
    (10.0, -12.0, -2.0), // crosses zero downward
    (-2.5, -4.0, -6.5),  // negative to more negative
    (-6.5, 10.0, 3.5),   // crosses zero upward
    (2.5, -2.5, 0.0),    // lands on zero
    (0.0, -0.75, -0.75), // leaves zero downward
];

#[test]
fn test_adjust_translates_along_the_whole_real_line() {
    for (start, shift, expected) in TRANSLATIONS {
        let mut t = MinkowskiTime::new(1, TimeScale::Second, start);
        let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
        grid.set(PointIndex::new1d(0), shift);

        assert!(t.adjust(&grid).is_ok(), "{start} + {shift}");
        assert_eq!(t.time_unit(), expected, "{start} + {shift}");
    }
}

#[test]
fn test_adjust_by_a_shift_and_its_negation_restores_the_time() {
    for (start, shift, _) in TRANSLATIONS {
        let mut t = MinkowskiTime::new(1, TimeScale::Second, start);
        let forward: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
        forward.set(PointIndex::new1d(0), shift);
        let back: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
        back.set(PointIndex::new1d(0), -shift);

        assert!(t.adjust(&forward).is_ok(), "{start} + {shift}");
        assert!(t.adjust(&back).is_ok(), "{start} + {shift} - {shift}");
        assert_eq!(t.time_unit(), start, "{start} ± {shift}");
    }
}

#[test]
fn test_adjust_by_zero_keeps_a_zero_time() {
    let mut t = MinkowskiTime::new(1, TimeScale::Second, 0.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), 0.0);

    assert!(t.adjust(&grid).is_ok());
    assert_eq!(t.time_unit(), 0.0);
}

#[test]
fn test_adjust_nan_input() {
    let mut t = MinkowskiTime::new(1, TimeScale::Second, 1.0);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), f64::NAN);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("NaN"));
    assert_eq!(t.time_unit(), 1.0);
}

#[test]
fn test_adjust_result_is_nan() {
    let mut t = MinkowskiTime::new(1, TimeScale::Second, f64::MAX);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), f64::INFINITY);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not finite"));
    assert_eq!(t.time_unit(), f64::MAX);
}

#[test]
fn test_adjust_refuses_an_overflow_toward_negative_infinity() {
    let mut t = MinkowskiTime::new(1, TimeScale::Second, f64::MIN);

    let grid: ArrayGrid<f64, 1, 1, 1, 1> = ArrayGrid::new(ArrayType::Array1D);
    grid.set(PointIndex::new1d(0), f64::MIN);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not finite"));
    assert_eq!(t.time_unit(), f64::MIN);
}
