/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::utils_test::test_utils_array_grid::{
    ARRAY_TYPES, get_slots_array_grid,
};
use deep_causality_context::*;
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

#[test]
fn test_tangent_spacetime_adjust_success() {
    let mut t = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);
    grid.set(PointIndex::new3d(0, 0, 3), 1.0);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);

    let result = t.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(t.x(), 2.0);
    assert_eq!(t.y(), 3.0);
    assert_eq!(t.z(), 4.0);
    assert_eq!(t.time_unit(), 5.0);
}

#[test]
fn test_tangent_spacetime_adjust_invalid_x_inf() {
    let mut t = TangentSpacetime::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "AdjustmentError: Adjustment failed, adjusted x value is not finite"
    );
}

#[test]
fn test_tangent_spacetime_adjust_invalid_y_inf() {
    let mut t = TangentSpacetime::new(1, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1, 0.1, 0.1);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 2), f64::INFINITY);

    let result = t.adjust(&grid);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "AdjustmentError: Adjustment failed, adjusted Y value is not finite"
    );
}

#[test]
fn test_tangent_spacetime_adjust_invalid_z_inf() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 3), f64::INFINITY);

    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_adjust_invalid_z_neg_inf() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 3), f64::NEG_INFINITY);

    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_adjust_invalid_z_nan() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 3), f64::NAN);

    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_adjust_invalid_t_inf() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::INFINITY);

    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_adjust_invalid_t_neg_inf() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::NEG_INFINITY);

    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_adjust_invalid_t_nan() {
    let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN);

    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_tangent_spacetime_adjust_adds_grid_positions_0_to_3_of_every_grid_to_t_x_y_z() {
    for array_type in ARRAY_TYPES {
        let mut s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.1, 0.1, 0.1);
        let grid = get_slots_array_grid(array_type, [40.0, 10.0, 20.0, 30.0]);

        assert!(s.adjust(&grid).is_ok(), "{array_type:?}");
        assert_eq!(
            [s.time_unit(), s.x(), s.y(), s.z()],
            [44.0, 11.0, 22.0, 33.0],
            "{array_type:?}"
        );
    }
}
