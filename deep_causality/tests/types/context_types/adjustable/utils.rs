// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use dcl_data_structures::grid_type::{ArrayGrid, ArrayType};
use dcl_data_structures::prelude::PointIndex;

pub const HEIGHT: usize = 1;
// set all unused dimensions to 0 to save some memory.
pub const WIDTH: usize = 0;
pub const DEPTH: usize = 0;
pub const TIME: usize = 0;

pub type AdjustmentData = ArrayGrid<i32, WIDTH, HEIGHT, DEPTH, TIME>;

pub fn get_array_grid(val: i32) -> AdjustmentData {
    let ag: ArrayGrid<i32, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(ArrayType::Array1D);

    // Create a 1D PointIndex
    let p = PointIndex::new1d(0);

    // Store an i32 with th position of the point index
    ag.set(p, val);

    ag
}
