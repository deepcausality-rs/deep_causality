// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};
use deep_causality::prelude::{Adjustable, AdjustableData};

const HEIGHT: usize = 1;
// set all unused dimensions to 0 to save some memory.
const WIDTH: usize = 0;
const DEPTH: usize = 0;
const TIME: usize = 0;

type AdjustmentData = ArrayGrid<i32, WIDTH, HEIGHT, DEPTH, TIME>;

fn get_array_grid(val: i32) -> AdjustmentData {
    let ag: ArrayGrid<i32, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(ArrayType::Array1D);

    // Create a 1D PointIndex
    let p = PointIndex::new1d(0);

    // Store an i32 with th position of the point index
    ag.set(p, val);

    ag
}

#[test]
fn test_update() {
    let mut d = AdjustableData::new(0, 0);
    assert_eq!(d.data(), 0);

    let array_grid = get_array_grid(42);

    let res = d.update(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.data(), 42);
}

#[test]
fn test_update_err() {
    let mut d = AdjustableData::new(0, 42);
    assert_eq!(d.data(), 42);

    let array_grid = get_array_grid(0);

    // Update fails with UpdateError
    let res = d.update(&array_grid);
    assert!(res.is_err());

    // Old value still in place, as before the failed update.
    assert_eq!(d.data(), 42);
}

#[test]
fn test_adjust() {
    let mut d = AdjustableData::new(0, 21);
    assert_eq!(d.data(), 21);

    let array_grid = get_array_grid(21);

    let res = d.adjust(&array_grid);
    assert!(res.is_ok());

    assert_eq!(d.data(), 42);
}

#[test]
fn test_adjust_err() {
    let mut d = AdjustableData::new(0, 21);
    assert_eq!(d.data(), 21);

    let array_grid = get_array_grid(-23);

    // adjustment fails with AdjustmentError
    let res = d.adjust(&array_grid);
    assert!(res.is_err());

    // Old value still in place, as before the failed adjustment.
    assert_eq!(d.data(), 21);
}