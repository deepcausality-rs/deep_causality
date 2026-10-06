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
fn test_ned_space_update_success() {
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
fn test_ned_space_update_north_fails_on_nan() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), f64::NAN); // North: Invalid adjustment
    grid.set(PointIndex::new3d(0, 0, 1), 0.0);
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);

    let result = ned.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ned_space_update_east_fails_on_nan() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0);
    grid.set(PointIndex::new3d(0, 0, 1), f64::NAN); // East: Invalid adjustment
    grid.set(PointIndex::new3d(0, 0, 2), 0.0);

    let result = ned.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ned_space_update_down_fails_on_nan() {
    let mut ned = NedSpace::new(1, 0.0, 0.0, 0.0);
    let grid: ArrayGrid<f64, 3, 3, 3, 1> = ArrayGrid::new(ArrayType::Array3D);

    grid.set(PointIndex::new3d(0, 0, 0), 0.0);
    grid.set(PointIndex::new3d(0, 0, 1), 0.0);
    grid.set(PointIndex::new3d(0, 0, 2), f64::NAN); // Down: Invalid adjustment

    let result = ned.update(&grid);
    assert!(result.is_err());
}

#[test]
fn test_ned_space_adjust_success() {
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

#[test]
fn test_ned_space_update_replaces_north_east_down_with_grid_positions_0_to_2_of_every_grid() {
    for array_type in ARRAY_TYPES {
        let mut ned = NedSpace::new(1, 1.0, 2.0, 3.0);
        let grid = get_slots_array_grid(array_type, [10.0, 20.0, 30.0]);

        assert!(ned.update(&grid).is_ok(), "{array_type:?}");
        assert_eq!(
            [ned.north(), ned.east(), ned.down()],
            [10.0, 20.0, 30.0],
            "{array_type:?}"
        );
    }
}

#[test]
fn test_ned_space_adjust_adds_grid_positions_0_to_2_of_every_grid_to_north_east_down() {
    for array_type in ARRAY_TYPES {
        let mut ned = NedSpace::new(1, 1.0, 2.0, 3.0);
        let grid = get_slots_array_grid(array_type, [10.0, 20.0, 30.0]);

        assert!(ned.adjust(&grid).is_ok(), "{array_type:?}");
        assert_eq!(
            [ned.north(), ned.east(), ned.down()],
            [11.0, 22.0, 33.0],
            "{array_type:?}"
        );
    }
}
