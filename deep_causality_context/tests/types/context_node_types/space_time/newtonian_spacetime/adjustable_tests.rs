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
fn test_update_success() {
    let mut s = NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), 10.0);
    grid.set(PointIndex::new3d(0, 0, 2), 20.0);
    grid.set(PointIndex::new3d(0, 0, 3), 30.0);
    grid.set(PointIndex::new3d(0, 0, 0), 40.0);

    let result = s.update(&grid);
    assert!(result.is_ok());
    assert_eq!(s.x(), 10.0);
    assert_eq!(s.y(), 20.0);
    assert_eq!(s.z(), 30.0);
    assert_eq!(s.time_unit(), 40.0);
}

#[test]
fn test_update_nan_should_fail() {
    let mut s = NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);
    grid.set(PointIndex::new3d(0, 0, 3), 1.0);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);

    let result = s.update(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not a finite value"));
}

#[test]
fn test_adjust_success() {
    let mut s = NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 1.0);
    grid.set(PointIndex::new3d(0, 0, 3), 1.0);
    grid.set(PointIndex::new3d(0, 0, 0), 1.0);

    let result = s.adjust(&grid);
    assert!(result.is_ok());
    assert_eq!(s.x(), 2.0);
    assert_eq!(s.y(), 3.0);
    assert_eq!(s.z(), 4.0);
    assert_eq!(s.time_unit(), 5.0);
}

#[test]
fn test_update_fails_with_non_finite_x() {
    let mut s = NewtonianSpacetime::new(1, f64::MAX, 1.0, 1.0, 1.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY);
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);
    grid.set(PointIndex::new3d(0, 0, 3), 0.0);
    grid.set(PointIndex::new3d(0, 0, 0), 0.0);

    let result = s.update(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not a finite value"));
}

#[test]
fn test_update_fails_with_non_finite_y() {
    let mut s = NewtonianSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0); // x
    grid.set(PointIndex::new3d(0, 0, 2), f64::NAN); // y (invalid)
    grid.set(PointIndex::new3d(0, 0, 3), 3.0); // z
    grid.set(PointIndex::new3d(0, 0, 0), 4.0); // t
    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_fails_with_non_finite_z() {
    let mut s = NewtonianSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 2.0);
    grid.set(PointIndex::new3d(0, 0, 3), f64::INFINITY); // z (invalid)
    grid.set(PointIndex::new3d(0, 0, 0), 4.0);
    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_fails_with_non_finite_t() {
    let mut s = NewtonianSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 2.0);
    grid.set(PointIndex::new3d(0, 0, 3), 3.0);
    grid.set(PointIndex::new3d(0, 0, 0), f64::NEG_INFINITY); // t (invalid)
    let result = s.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_fails_with_non_finite_x() {
    let mut s = NewtonianSpacetime::new(1, f64::MAX, 1.0, 1.0, 1.0, TimeScale::Second);

    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), f64::INFINITY);
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);
    grid.set(PointIndex::new3d(0, 0, 3), 0.0);
    grid.set(PointIndex::new3d(0, 0, 0), 0.0);

    let result = s.adjust(&grid);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not a finite value"));
}

#[test]
fn test_adjust_fails_with_non_finite_y() {
    let mut s = NewtonianSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0); // x
    grid.set(PointIndex::new3d(0, 0, 2), f64::NAN); // y (invalid)
    grid.set(PointIndex::new3d(0, 0, 3), 3.0); // z
    grid.set(PointIndex::new3d(0, 0, 0), 4.0); // t
    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_fails_with_non_finite_z() {
    let mut s = NewtonianSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 2.0);
    grid.set(PointIndex::new3d(0, 0, 3), f64::INFINITY); // z (invalid)
    grid.set(PointIndex::new3d(0, 0, 0), 4.0);
    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_adjust_fails_with_non_finite_t() {
    let mut s = NewtonianSpacetime::new(0, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array3D);
    grid.set(PointIndex::new3d(0, 0, 1), 1.0);
    grid.set(PointIndex::new3d(0, 0, 2), 2.0);
    grid.set(PointIndex::new3d(0, 0, 3), 3.0);
    grid.set(PointIndex::new3d(0, 0, 0), f64::NEG_INFINITY); // t (invalid)
    let result = s.adjust(&grid);
    assert!(result.is_err());
}

#[test]
fn test_update_replaces_t_x_y_z_with_grid_positions_0_to_3_of_every_grid() {
    for array_type in ARRAY_TYPES {
        let mut s = NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
        let grid = get_slots_array_grid(array_type, [40.0, 10.0, 20.0, 30.0]);

        assert!(s.update(&grid).is_ok(), "{array_type:?}");
        assert_eq!(
            [s.time_unit(), s.x(), s.y(), s.z()],
            [40.0, 10.0, 20.0, 30.0],
            "{array_type:?}"
        );
    }
}

#[test]
fn test_adjust_adds_grid_positions_0_to_3_of_every_grid_to_t_x_y_z() {
    for array_type in ARRAY_TYPES {
        let mut s = NewtonianSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
        let grid = get_slots_array_grid(array_type, [40.0, 10.0, 20.0, 30.0]);

        assert!(s.adjust(&grid).is_ok(), "{array_type:?}");
        assert_eq!(
            [s.time_unit(), s.x(), s.y(), s.z()],
            [44.0, 11.0, 22.0, 33.0],
            "{array_type:?}"
        );
    }
}
