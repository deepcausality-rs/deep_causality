// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::{ArrayGrid, ArrayType, Grid, PointIndex};

#[test]
fn test_grid_storage_dimensions() {
    const W: usize = 2;
    const H: usize = 3;
    const D: usize = 4;
    const T: usize = 5;

    // Test 1D Grid
    let grid_1d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array1D);
    match grid_1d {
        ArrayGrid::ArrayGrid1D(g) => {
            assert_eq!(g.width(), None);
            assert_eq!(g.height(), Some(&H));
            assert_eq!(g.depth(), None);
            assert_eq!(g.time(), None);
        }
        _ => panic!("Wrong grid type"),
    }

    // Test 2D Grid
    let grid_2d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array2D);
    match grid_2d {
        ArrayGrid::ArrayGrid2D(g) => {
            assert_eq!(g.width(), Some(&W));
            assert_eq!(g.height(), Some(&H));
            assert_eq!(g.depth(), None);
            assert_eq!(g.time(), None);
        }
        _ => panic!("Wrong grid type"),
    }

    // Test 3D Grid
    let grid_3d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array3D);
    match grid_3d {
        ArrayGrid::ArrayGrid3D(g) => {
            assert_eq!(g.width(), Some(&W));
            assert_eq!(g.height(), Some(&H));
            assert_eq!(g.depth(), Some(&D));
            assert_eq!(g.time(), None);
        }
        _ => panic!("Wrong grid type"),
    }

    // Test 4D Grid
    let grid_4d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array4D);
    match grid_4d {
        ArrayGrid::ArrayGrid4D(g) => {
            assert_eq!(g.width(), Some(&W));
            assert_eq!(g.height(), Some(&H));
            assert_eq!(g.depth(), Some(&D));
            assert_eq!(g.time(), Some(&T));
        }
        _ => panic!("Wrong grid type"),
    }
}

#[test]
fn test_grid_storage_operations() {
    const W: usize = 2;
    const H: usize = 3;
    const D: usize = 4;
    const T: usize = 5;

    // Test operations on 4D grid to cover all dimensions
    let mut grid: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array4D);

    // Test setting and getting values at various points
    let test_points = [
        PointIndex::new1(1),
        PointIndex::new2(1, 2),
        PointIndex::new3(1, 2, 3),
        PointIndex::new4(1, 2, 3, 4),
    ];

    for (i, point) in test_points.iter().enumerate() {
        let value = i as i32;
        grid.set(*point, value);
        assert_eq!(grid.get(*point), Some(&value));
    }

    // Test bounds checking
    let invalid_points = [
        PointIndex::new1(H + 1),
        PointIndex::new2(W + 1, H),
        PointIndex::new3(W, H + 1, D),
        PointIndex::new4(W, H, D + 1, T),
    ];

    for point in invalid_points.iter() {
        assert!(grid.get(*point).is_none());
    }
}

#[test]
fn test_grid_default_values() {
    const W: usize = 2;
    const H: usize = 2;
    const D: usize = 2;
    const T: usize = 2;

    // Test that new grids are initialized with default values
    let grids = [
        ArrayGrid::<i32, W, H, D, T>::new(ArrayType::Array1D),
        ArrayGrid::<i32, W, H, D, T>::new(ArrayType::Array2D),
        ArrayGrid::<i32, W, H, D, T>::new(ArrayType::Array3D),
        ArrayGrid::<i32, W, H, D, T>::new(ArrayType::Array4D),
    ];

    for grid in grids.iter() {
        let point = PointIndex::new4(0, 0, 0, 0);
        assert_eq!(grid.get(point), Some(&0)); // 0 is the default value for i32
    }
}
