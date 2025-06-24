// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use dcl_data_structures::grid_type::{ArrayGrid, ArrayType};
use dcl_data_structures::prelude::PointIndex;

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

pub fn _get_3d_array_grid(v1: i32, v2: i32, v3: i32) -> AdjustmentData {
    let array_type = ArrayType::Array3D;
    let ag: ArrayGrid<i32, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    // Create a 3D PointIndex for each of the updated x,y,z coordinates
    let p1 = PointIndex::new3d(0, 0, 0);
    let p2 = PointIndex::new3d(0, 0, 1);
    let p3 = PointIndex::new3d(0, 0, 2);

    // Store an i32 with th position of the point index
    ag.set(p1, v1);
    ag.set(p2, v2);
    ag.set(p3, v3);

    ag
}

pub fn _get_4d_array_grid(v1: i32, v2: i32, v3: i32, t: i32) -> AdjustmentData {
    let array_type = ArrayType::Array4D;
    let ag: ArrayGrid<i32, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    // Create a 4D PointIndex for each of the updated x,y,z coordinates plus time t
    let p1 = PointIndex::new4d(0, 0, 0, 0);
    let p2 = PointIndex::new4d(0, 0, 0, 1);
    let p3 = PointIndex::new4d(0, 0, 0, 2);
    let pt = PointIndex::new4d(0, 0, 0, 3);

    // Store an i32 with th position of the point index
    ag.set(p1, v1);
    ag.set(p2, v2);
    ag.set(p3, v3);
    ag.set(pt, t);

    ag
}
