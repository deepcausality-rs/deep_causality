// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use dcl_data_structures::prelude::{ArrayGrid, Grid, PointIndex};
use dcl_data_structures::prelude::ArrayType::{Array1D, Array2D, Array3D, Array4D};

const WIDTH: usize = 5;
const HEIGHT: usize = 5;
const DEPTH: usize = 5;
const TIME: usize = 5;

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
fn test_array_grid_array_1d() {
    let array_type = Array1D;
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    let p = PointIndex::new1d(1);
    ag.set(p, 1);

    let res = ag.get(p);
    assert_eq!(res, 1);

    let g: &Grid<[usize; HEIGHT], usize> = ag.array_grid_1d().expect("failed to get array grid");
    assert_eq!(g.height().unwrap(), HEIGHT);
}


#[test]
fn test_array_grid_array_2d() {
    let array_type = Array2D;
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    let p = PointIndex::new2d(1, 2);
    ag.set(p, 2);

    let res = ag.get(p);
    let exp = 2;
    assert_eq!(res, exp);

    let g = ag.array_grid_2d().expect("failed to create array grid");
    assert_eq!(g.height().unwrap(), HEIGHT);
}


#[test]
fn test_array_grid_array_3d() {
    let array_type = Array3D;
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    let p = PointIndex::new3d(1, 2, 3);
    ag.set(p, 3);
    let res = ag.get(p);
    let exp = 3;
    assert_eq!(res, exp);

    let g = ag.array_grid_3d()
        .expect("failed to create array grid");

    assert_eq!(g.height().unwrap(), HEIGHT);
    assert_eq!(g.width().unwrap(), WIDTH);
    assert_eq!(g.depth().unwrap(), DEPTH);
}


#[test]
fn test_array_grid_array_4d() {
    let array_type = Array4D;
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);
    let g = ag.array_grid_4d()
        .expect("failed to create array grid");

    assert_eq!(g.height().unwrap(), HEIGHT);
    assert_eq!(g.width().unwrap(), WIDTH);
    assert_eq!(g.depth().unwrap(), DEPTH);
    assert_eq!(g.time().unwrap(), TIME);

    let p = PointIndex::new4d(1, 2, 1, 1);
    g.set(p, 4);
    let res = g.get(p);
    let exp = 4;
    assert_eq!(res, exp);
}