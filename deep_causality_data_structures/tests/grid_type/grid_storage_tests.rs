/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

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
            assert_eq!(g.height().unwrap(), H);
            assert_eq!(g.depth(), None);
            assert_eq!(g.time(), None);
        }
        _ => panic!("Wrong grid type"),
    }

    // Test 2D Grid
    let grid_2d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array2D);
    match grid_2d {
        ArrayGrid::ArrayGrid2D(g) => {
            assert_eq!(g.width().unwrap(), W);
            assert_eq!(g.height().unwrap(), H);
            assert_eq!(g.depth(), None);
            assert_eq!(g.time(), None);
        }
        _ => panic!("Wrong grid type"),
    }

    // Test 3D Grid
    let grid_3d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array3D);
    match grid_3d {
        ArrayGrid::ArrayGrid3D(g) => {
            assert_eq!(g.width().unwrap(), W);
            assert_eq!(g.height().unwrap(), H);
            assert_eq!(g.depth().unwrap(), D);
            assert_eq!(g.time(), None);
        }
        _ => panic!("Wrong grid type"),
    }

    // Test 4D Grid
    let grid_4d: ArrayGrid<i32, W, H, D, T> = ArrayGrid::new(ArrayType::Array4D);
    match grid_4d {
        ArrayGrid::ArrayGrid4D(g) => {
            assert_eq!(g.width().unwrap(), W);
            assert_eq!(g.height().unwrap(), H);
            assert_eq!(g.depth().unwrap(), D);
            assert_eq!(g.time().unwrap(), T);
        }
        _ => panic!("Wrong grid type"),
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
        let point = PointIndex::new4d(0, 0, 0, 0);
        assert_eq!(grid.get(point), 0); // 0 is the default value for i32
    }
}
