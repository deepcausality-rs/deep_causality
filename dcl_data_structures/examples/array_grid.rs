// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};

// Consts dimensions requires for const generic parameters
// Use these to check whether your PointIndex stays within the Array boundaries.
const WIDTH: usize = 5;
const HEIGHT: usize = 5;
const DEPTH: usize = 5;
const TIME: usize = 5;

// Util function that helps with type inference.
fn get_array_grid<T: Copy + Default>(array_type: ArrayType) -> ArrayGrid<T, WIDTH, HEIGHT, DEPTH, TIME> {
    ArrayGrid::new(array_type)
}

pub fn main() {

    // Make a simple 1D Array of type usize
    let array_type = ArrayType::Array1D;
    let ag = get_array_grid(array_type);

    // Create a 1D PointIndex
    let p = PointIndex::new1d(1);

    // Store a usize with the point index
    ag.set(p, 42);

    // Get the usize for the point index
    let res = ag.get(p);
    assert_eq!(res, 42);

    // Make a 3D array aka matrix over x,y,z that stores u64
    // Notice, only the ArrayType changes to do that.
    // Also, notice the target type (u64) is always the first generic parameter
    let array_type = ArrayType::Array3D;
    let ag = get_array_grid(array_type);

    // Create a new 3D point index
    let p = PointIndex::new3d(1, 2, 3);

    // Set a u64 value
    ag.set(p, 3);

    // Get the value at the point index
    let res = ag.get(p);
    let exp = 3;
    assert_eq!(res, exp);


    // ArrayGrid requires Copy + Default to store MyStuct
    #[derive(Debug, Default, Copy, Clone)]
    struct MyStruct {
        number: usize,
        mod_five: bool,
    }

    // Make a 4D array aka matrix over x,y,z that stores My struct
    // Notice, only the ArrayType changes to do that.
    let array_type = ArrayType::Array4D;
    let ag: ArrayGrid<MyStruct, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    // Create a new 4D point index where only time varies
    let idx_t0 = PointIndex::new4d(1, 1, 1, 0);
    let idx_t1 = PointIndex::new4d(1, 1, 1, 1);
    let idx_t2 = PointIndex::new4d(1, 1, 1, 2);

    // Create some data for each index
    let my_struct_t0 = MyStruct { number: 23, mod_five: false };
    let my_struct_t1 = MyStruct { number: 24, mod_five: false };
    let my_struct_t2 = MyStruct { number: 25, mod_five: true };

    // Store data
    ag.set(idx_t0, my_struct_t0);
    ag.set(idx_t1, my_struct_t1);
    ag.set(idx_t2, my_struct_t2);

    // Get data at t2
    let res = ag.get(idx_t2);

    // Verify results
    let exp_number = 25;
    assert_eq!(res.number, exp_number);
    let exp_mod = true;
    assert_eq!(res.mod_five, exp_mod);
}