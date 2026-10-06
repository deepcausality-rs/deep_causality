/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

pub const HEIGHT: usize = 5;
// set all unused dimensions to 0 to save some memory.
pub const WIDTH: usize = 5;
pub const DEPTH: usize = 5;
pub const TIME: usize = 5;

pub type AdjustmentData = ArrayGrid<i32, WIDTH, HEIGHT, DEPTH, TIME>;

pub fn get_1d_array_grid(val: i32) -> AdjustmentData {
    let array_type = ArrayType::Array1D;
    let ag: ArrayGrid<i32, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    // Create a 1D PointIndex
    let p = PointIndex::new1d(0);

    // Store an i32 with th position of the point index
    ag.set(p, val);

    ag
}

/// The four grid dimensionalities, 1D to 4D.
pub const ARRAY_TYPES: [ArrayType; 4] = [
    ArrayType::Array1D,
    ArrayType::Array2D,
    ArrayType::Array3D,
    ArrayType::Array4D,
];

/// A 4×4×4×4 grid of `array_type` whose positions 0 to `N - 1` hold `values`. Position `k` is
/// `PointIndex::new1d(k)` of a 1D grid, `PointIndex::new2d(k, 0)` of a 2D grid,
/// `PointIndex::new3d(0, 0, k)` of a 3D grid and `PointIndex::new4d(0, 0, k, 0)` of a 4D grid.
/// Every other entry is zero.
///
/// `N` is at most 4, the length of each axis:
///
/// ```
/// use deep_causality_context::utils_test::test_utils_array_grid::get_slots_array_grid;
/// use deep_causality_data_structures::ArrayType;
///
/// get_slots_array_grid(ArrayType::Array1D, [1.0, 2.0, 3.0, 4.0]);
/// ```
///
/// A fifth value does not compile:
///
/// ```compile_fail
/// use deep_causality_context::utils_test::test_utils_array_grid::get_slots_array_grid;
/// use deep_causality_data_structures::ArrayType;
///
/// get_slots_array_grid(ArrayType::Array1D, [1.0, 2.0, 3.0, 4.0, 5.0]);
/// ```
pub fn get_slots_array_grid<const N: usize>(
    array_type: ArrayType,
    values: [f64; N],
) -> ArrayGrid<f64, 4, 4, 4, 4> {
    const {
        assert!(
            N <= 4,
            "a 4×4×4×4 grid holds at most four positions along an axis"
        )
    };
    let grid = ArrayGrid::new(array_type);
    values.iter().enumerate().for_each(|(k, &value)| {
        let position = match array_type {
            ArrayType::Array1D => PointIndex::new1d(k),
            ArrayType::Array2D => PointIndex::new2d(k, 0),
            ArrayType::Array3D => PointIndex::new3d(0, 0, k),
            ArrayType::Array4D => PointIndex::new4d(0, 0, k, 0),
        };
        grid.set(position, value);
    });
    grid
}
