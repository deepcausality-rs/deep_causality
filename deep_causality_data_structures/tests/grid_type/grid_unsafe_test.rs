/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[cfg(feature = "unsafe")]
mod tests {
    use deep_causality_data_structures::{Grid, PointIndex};
    use std::sync::atomic::{AtomicBool, Ordering};

    const SIZE_1D: usize = 5;
    const SIZE_2D: usize = 3;
    const SIZE_3D: usize = 2;
    const SIZE_4D: usize = 2;

    #[test]
    fn test_grid_1d_creation() {
        let storage = [0u32; SIZE_1D];
        let grid: Grid<[u32; SIZE_1D], u32> = Grid::new(storage);

        assert_eq!(grid.width(), None);
        assert_eq!(grid.height(), Some(SIZE_1D));
        assert_eq!(grid.depth(), None);
        assert_eq!(grid.time(), None);
    }

    #[test]
    fn test_grid_2d_creation() {
        let storage = [[0u32; SIZE_2D]; SIZE_2D];
        let grid: Grid<[[u32; SIZE_2D]; SIZE_2D], u32> = Grid::new(storage);

        assert_eq!(grid.width(), Some(SIZE_2D));
        assert_eq!(grid.height(), Some(SIZE_2D));
        assert_eq!(grid.depth(), None);
        assert_eq!(grid.time(), None);
    }

    #[test]
    fn test_grid_3d_creation() {
        let storage = [[[0u32; SIZE_3D]; SIZE_3D]; SIZE_3D];
        let grid: Grid<[[[u32; SIZE_3D]; SIZE_3D]; SIZE_3D], u32> = Grid::new(storage);

        assert_eq!(grid.width(), Some(SIZE_3D));
        assert_eq!(grid.height(), Some(SIZE_3D));
        assert_eq!(grid.depth(), Some(SIZE_3D));
        assert_eq!(grid.time(), None);
    }

    #[test]
    fn test_grid_4d_creation() {
        let storage = [[[[0u32; SIZE_4D]; SIZE_4D]; SIZE_4D]; SIZE_4D];
        let grid: Grid<[[[[u32; SIZE_4D]; SIZE_4D]; SIZE_4D]; SIZE_4D], u32> = Grid::new(storage);

        assert_eq!(grid.width(), Some(SIZE_4D));
        assert_eq!(grid.height(), Some(SIZE_4D));
        assert_eq!(grid.depth(), Some(SIZE_4D));
        assert_eq!(grid.time(), Some(SIZE_4D));
    }

    #[test]
    fn test_get_default_value() {
        let storage = [42u32; SIZE_1D];
        let grid: Grid<[u32; SIZE_1D], u32> = Grid::new(storage);

        // First verify that we get the actual value when initialized
        let point = PointIndex::new1d(2);
        let value = grid.get(point);
        assert_eq!(value, 42u32);

        // Now test the default value path by setting initialized to false
        unsafe {
            let grid_ptr =
                &grid as *const Grid<[u32; SIZE_1D], u32> as *mut Grid<[u32; SIZE_1D], u32>;
            let storage_offset = 0;
            let storage_size = std::mem::size_of::<[u32; SIZE_1D]>();
            let initialized_ptr =
                (grid_ptr as *mut u8).add(storage_offset + storage_size) as *mut AtomicBool;
            (*initialized_ptr).store(false, Ordering::Release);
        }

        let default_value = grid.get(point);
        assert_eq!(default_value, u32::default());
    }

    #[test]
    fn test_set_and_get_1d() {
        let storage = [0u32; SIZE_1D];
        let grid: Grid<[u32; SIZE_1D], u32> = Grid::new(storage);

        let point = PointIndex::new1d(2);
        let value = 42u32;

        grid.set(point, value);
        let result = grid.get(point);

        assert_eq!(result, value);
    }

    #[test]
    fn test_set_and_get_2d() {
        let storage = [[0u32; SIZE_2D]; SIZE_2D];
        let grid: Grid<[[u32; SIZE_2D]; SIZE_2D], u32> = Grid::new(storage);

        let point = PointIndex::new2d(1, 2);
        let value = 42u32;

        grid.set(point, value);
        let result = grid.get(point);

        assert_eq!(result, value);
    }

    #[test]
    fn test_set_and_get_3d() {
        let storage = [[[0u32; SIZE_3D]; SIZE_3D]; SIZE_3D];
        let grid: Grid<[[[u32; SIZE_3D]; SIZE_3D]; SIZE_3D], u32> = Grid::new(storage);

        let point = PointIndex::new3d(1, 1, 1);
        let value = 42u32;

        grid.set(point, value);
        let result = grid.get(point);

        assert_eq!(result, value);
    }

    #[test]
    fn test_set_and_get_4d() {
        let storage = [[[[0u32; SIZE_4D]; SIZE_4D]; SIZE_4D]; SIZE_4D];
        let grid: Grid<[[[[u32; SIZE_4D]; SIZE_4D]; SIZE_4D]; SIZE_4D], u32> = Grid::new(storage);

        let point = PointIndex::new4d(1, 1, 1, 1);
        let value = 42u32;

        grid.set(point, value);
        let result = grid.get(point);

        assert_eq!(result, value);
    }

    #[test]
    fn test_multiple_set_and_get() {
        let storage = [[0u32; SIZE_2D]; SIZE_2D];
        let grid: Grid<[[u32; SIZE_2D]; SIZE_2D], u32> = Grid::new(storage);

        let points = [
            PointIndex::new2d(0, 0),
            PointIndex::new2d(1, 1),
            PointIndex::new2d(2, 2),
        ];
        let values = [1u32, 2u32, 3u32];

        // Set multiple values
        for (point, &value) in points.iter().zip(values.iter()) {
            grid.set(*point, value);
        }

        // Verify all values were set correctly
        for (point, &expected) in points.iter().zip(values.iter()) {
            let result = grid.get(*point);
            assert_eq!(result, expected);
        }
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_out_of_bounds_2d() {
        let storage = [[0u32; SIZE_2D]; SIZE_2D];
        let grid: Grid<[[u32; SIZE_2D]; SIZE_2D], u32> = Grid::new(storage);

        let point = PointIndex::new2d(SIZE_2D, SIZE_2D);
        grid.set(point, 42);
    }

    #[test]
    fn test_overwrite_value() {
        let storage = [0u32; SIZE_1D];
        let grid: Grid<[u32; SIZE_1D], u32> = Grid::new(storage);

        let point = PointIndex::new1d(2);

        grid.set(point, 42);
        assert_eq!(grid.get(point), 42);

        grid.set(point, 24);
        assert_eq!(grid.get(point), 24);
    }

    #[test]
    fn test_1d_grid() {
        let array = [0i32; SIZE_1D];
        let grid = Grid::<[i32; SIZE_1D], i32>::new(array);

        let point = PointIndex::new1d(0);
        assert_eq!(grid.get(point), 0);

        // SAFETY: This is safe because we're using atomic operations to ensure thread safety
        unsafe {
            let grid_ptr =
                &grid as *const Grid<[i32; SIZE_1D], i32> as *mut Grid<[i32; SIZE_1D], i32>;
            let storage_offset = 0;
            let storage_ptr = (grid_ptr as *mut u8).add(storage_offset) as *mut [i32; SIZE_1D];
            (*storage_ptr)[0] = 42;
        }

        assert_eq!(grid.get(point), 42);

        grid.set(point, 24);
        assert_eq!(grid.get(point), 24);
    }

    #[test]
    fn test_set_get() {
        let array = [0i32; SIZE_1D];
        let grid = Grid::<[i32; SIZE_1D], i32>::new(array);

        let point = PointIndex::new1d(0);
        assert_eq!(grid.get(point), 0);

        // SAFETY: This is safe because we're using atomic operations to ensure thread safety
        unsafe {
            let grid_ptr =
                &grid as *const Grid<[i32; SIZE_1D], i32> as *mut Grid<[i32; SIZE_1D], i32>;
            let storage_offset = 0;
            let storage_ptr = (grid_ptr as *mut u8).add(storage_offset) as *mut [i32; SIZE_1D];
            (*storage_ptr)[0] = 42;
        }

        assert_eq!(grid.get(point), 42);

        grid.set(point, 24);
        assert_eq!(grid.get(point), 24);
    }

    #[test]
    fn test_uninitialized_grid_returns_default() {
        let array = [0i32; SIZE_1D];
        let grid = Grid::<[i32; SIZE_1D], i32>::new(array);

        // SAFETY: This is safe because we're using atomic operations to ensure thread safety
        unsafe {
            let grid_ptr =
                &grid as *const Grid<[i32; SIZE_1D], i32> as *mut Grid<[i32; SIZE_1D], i32>;
            let storage_offset = 0;
            let storage_size = std::mem::size_of::<[i32; SIZE_1D]>();
            let initialized_ptr =
                (grid_ptr as *mut u8).add(storage_offset + storage_size) as *mut AtomicBool;
            (*initialized_ptr).store(false, Ordering::SeqCst);
        }

        // Test that uninitialized grid returns default value
        let point = PointIndex::new1d(0);
        assert_eq!(grid.get(point), i32::default());
    }
}
