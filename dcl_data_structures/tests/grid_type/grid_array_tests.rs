/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, Grid, PointIndex};

const WIDTH: usize = 5;
const HEIGHT: usize = 5;
const DEPTH: usize = 5;
const TIME: usize = 5;

fn get_array_grid<T: Copy + Default>(
    array_type: ArrayType,
) -> ArrayGrid<T, WIDTH, HEIGHT, DEPTH, TIME> {
    ArrayGrid::new(array_type)
}

#[test]
fn test_array_grid_array_1d() {
    let array_type = ArrayType::Array1D;
    let ag = get_array_grid(array_type);

    let p = PointIndex::new1d(1);
    ag.set(p, 1);

    let res = ag.get(p);
    assert_eq!(res, 1);

    let g = ag.array_grid_1d().expect("failed to get 1D array grid");

    let height = g.height().unwrap();
    assert_eq!(height, HEIGHT);
}

#[test]
fn test_array_grid_array_2d() {
    let array_type = ArrayType::Array2D;
    let ag = get_array_grid(array_type);

    let p = PointIndex::new2d(1, 2);
    ag.set(p, 2);

    let res = ag.get(p);
    let exp = 2;
    assert_eq!(res, exp);

    let g = ag.array_grid_2d().expect("failed to get 2D array grid");

    let height = g.height().unwrap();
    assert_eq!(height, HEIGHT);

    let width = g.width().unwrap();
    assert_eq!(width, WIDTH);
}

#[test]
fn test_array_grid_array_3d() {
    let array_type = ArrayType::Array3D;
    let ag = get_array_grid(array_type);

    let p = PointIndex::new3d(1, 2, 3);
    ag.set(p, 3);
    let res = ag.get(p);
    let exp = 3;
    assert_eq!(res, exp);

    let g = ag.array_grid_3d().expect("failed to get 3D array grid");

    let height = g.height().unwrap();
    assert_eq!(height, HEIGHT);

    let width = g.width().unwrap();
    assert_eq!(width, WIDTH);

    let depth = g.depth().unwrap();
    assert_eq!(depth, DEPTH);
}

#[test]
fn test_array_grid_array_4d() {
    let array_type = ArrayType::Array4D;
    let ag = get_array_grid(array_type);

    let p = PointIndex::new4d(1, 2, 1, 1);
    ag.set(p, 4);
    let res = ag.get(p);
    let exp = 4;
    assert_eq!(res, exp);

    let g = ag.array_grid_4d().expect("failed to create array grid");

    let height = g.height().unwrap();
    assert_eq!(height, HEIGHT);

    let width = g.width().unwrap();
    assert_eq!(width, WIDTH);

    let depth = g.depth().unwrap();
    assert_eq!(depth, DEPTH);

    let time = g.time().unwrap();
    assert_eq!(time, TIME);
}

#[test]
fn test_array_grid_1d() {
    let storage = [0.0f64; HEIGHT];
    let g = Grid::new(storage);
    assert_eq!(g.height().unwrap(), HEIGHT);
    assert_eq!(g.width(), None);

    let p = PointIndex::new1d(1);
    g.set(p, 1.0);

    let res = g.get(p);
    assert_eq!(res, 1.0f64);
}

#[test]
fn test_array_grid_2d() {
    let storage = [[0.0f64; WIDTH]; HEIGHT];
    let g: Grid<[[f64; WIDTH]; HEIGHT], f64> = Grid::new(storage);
    assert_eq!(g.height().unwrap(), HEIGHT);
    assert_eq!(g.width().unwrap(), WIDTH);

    let p = PointIndex::new2d(1, 1);
    g.set(p, 42.0f64);

    let res = g.get(p);
    assert_eq!(res, 42.0f64);
}

#[test]
fn test_array_grid_3d() {
    let storage = [[[0u8; WIDTH]; HEIGHT]; DEPTH];
    let g: Grid<[[[u8; WIDTH]; HEIGHT]; DEPTH], u8> = Grid::new(storage);
    assert_eq!(g.height().unwrap(), HEIGHT);
    assert_eq!(g.width().unwrap(), WIDTH);
    assert_eq!(g.depth().unwrap(), DEPTH);

    let p = PointIndex::new3d(1, 1, 1);
    g.set(p, 42);

    let res = g.get(p);
    assert_eq!(res, 42);
}

#[test]
fn test_array_grid_4d() {
    let storage = [[[[0u32; WIDTH]; HEIGHT]; DEPTH]; TIME];
    let g: Grid<[[[[u32; WIDTH]; HEIGHT]; DEPTH]; TIME], u32> = Grid::new(storage);
    assert_eq!(g.height().unwrap(), HEIGHT);
    assert_eq!(g.width().unwrap(), WIDTH);
    assert_eq!(g.depth().unwrap(), DEPTH);
    assert_eq!(g.time().unwrap(), TIME);

    let p = PointIndex::new4d(1, 1, 1, 1);
    g.set(p, 23);

    let res = g.get(p);
    assert_eq!(res, 23);
}

#[test]
fn test_array_grid_display() {
    const W: usize = 2;
    const H: usize = 3;
    const D: usize = 4;
    const T: usize = 5;

    // Test 1D Grid Display
    let grid_1d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array1D);
    let display_str = format!("{grid_1d}");
    assert!(display_str.contains("ArrayGrid1D"));

    // Test 2D Grid Display
    let grid_2d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array2D);
    let display_str = format!("{grid_2d}");
    assert!(display_str.contains("ArrayGrid2D"));

    // Test 3D Grid Display
    let grid_3d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array3D);
    let display_str = format!("{grid_3d}");
    assert!(display_str.contains("ArrayGrid3D"));

    // Test 4D Grid Display
    let grid_4d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array4D);
    let display_str = format!("{grid_4d}");
    assert!(display_str.contains("ArrayGrid4D"));
}
