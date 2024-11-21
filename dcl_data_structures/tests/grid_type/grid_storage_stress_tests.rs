// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::{Grid1D, Grid2D, Grid3D, Grid4D};

#[test]
fn test_grid1d_edge_cases() {
    // Test empty grid
    let grid = Grid1D::<i32>::new(0);
    assert_eq!(grid.len(), 0);
    assert!(grid.is_empty());
    
    // Test single element grid
    let mut grid = Grid1D::<i32>::new(1);
    grid.set(0, 42);
    assert_eq!(grid.get(0), Some(&42));
    
    // Test out of bounds access
    let grid = Grid1D::<i32>::new(5);
    assert_eq!(grid.get(5), None);
    assert_eq!(grid.get(usize::MAX), None);
}

#[test]
fn test_grid2d_edge_cases() {
    // Test empty grid
    let grid = Grid2D::<i32>::new(0, 0);
    assert_eq!(grid.len(), 0);
    assert!(grid.is_empty());
    
    // Test single element grid
    let mut grid = Grid2D::<i32>::new(1, 1);
    grid.set(0, 0, 42);
    assert_eq!(grid.get(0, 0), Some(&42));
    
    // Test asymmetric dimensions
    let grid = Grid2D::<i32>::new(2, 3);
    assert_eq!(grid.dim_x(), 2);
    assert_eq!(grid.dim_y(), 3);
    
    // Test out of bounds access
    assert_eq!(grid.get(2, 0), None);
    assert_eq!(grid.get(0, 3), None);
}

#[test]
fn test_grid3d_edge_cases() {
    // Test empty grid
    let grid = Grid3D::<i32>::new(0, 0, 0);
    assert_eq!(grid.len(), 0);
    assert!(grid.is_empty());
    
    // Test single element grid
    let mut grid = Grid3D::<i32>::new(1, 1, 1);
    grid.set(0, 0, 0, 42);
    assert_eq!(grid.get(0, 0, 0), Some(&42));
    
    // Test asymmetric dimensions
    let grid = Grid3D::<i32>::new(2, 3, 4);
    assert_eq!(grid.dim_x(), 2);
    assert_eq!(grid.dim_y(), 3);
    assert_eq!(grid.dim_z(), 4);
    
    // Test out of bounds access
    assert_eq!(grid.get(2, 0, 0), None);
    assert_eq!(grid.get(0, 3, 0), None);
    assert_eq!(grid.get(0, 0, 4), None);
}

#[test]
fn test_grid4d_edge_cases() {
    // Test empty grid
    let grid = Grid4D::<i32>::new(0, 0, 0, 0);
    assert_eq!(grid.len(), 0);
    assert!(grid.is_empty());
    
    // Test single element grid
    let mut grid = Grid4D::<i32>::new(1, 1, 1, 1);
    grid.set(0, 0, 0, 0, 42);
    assert_eq!(grid.get(0, 0, 0, 0), Some(&42));
    
    // Test asymmetric dimensions
    let grid = Grid4D::<i32>::new(2, 3, 4, 5);
    assert_eq!(grid.dim_x(), 2);
    assert_eq!(grid.dim_y(), 3);
    assert_eq!(grid.dim_z(), 4);
    assert_eq!(grid.dim_w(), 5);
    
    // Test out of bounds access
    assert_eq!(grid.get(2, 0, 0, 0), None);
    assert_eq!(grid.get(0, 3, 0, 0), None);
    assert_eq!(grid.get(0, 0, 4, 0), None);
    assert_eq!(grid.get(0, 0, 0, 5), None);
}

#[test]
fn test_grid_memory_behavior() {
    // Test with heap-allocated type
    let mut grid = Grid2D::<String>::new(2, 2);
    grid.set(0, 0, "test1".to_string());
    grid.set(0, 1, "test2".to_string());
    grid.set(1, 0, "test3".to_string());
    grid.set(1, 1, "test4".to_string());
    
    // Verify data is correctly stored
    assert_eq!(grid.get(0, 0), Some(&"test1".to_string()));
    assert_eq!(grid.get(0, 1), Some(&"test2".to_string()));
    assert_eq!(grid.get(1, 0), Some(&"test3".to_string()));
    assert_eq!(grid.get(1, 1), Some(&"test4".to_string()));
}

#[test]
fn test_grid_performance() {
    use std::time::Instant;
    
    // Test large grid operations
    let size = 100;
    let mut grid = Grid3D::<i32>::new(size, size, size);
    
    // Measure write performance
    let start = Instant::now();
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                grid.set(x, y, z, 42);
            }
        }
    }
    let write_duration = start.elapsed();
    
    // Measure read performance
    let start = Instant::now();
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                assert_eq!(grid.get(x, y, z), Some(&42));
            }
        }
    }
    let read_duration = start.elapsed();
    
    println!("Grid3D Write Duration: {:?}", write_duration);
    println!("Grid3D Read Duration: {:?}", read_duration);
}

#[test]
fn test_grid_concurrent_access() {
    use std::sync::Arc;
    use std::thread;
    
    let grid = Arc::new(std::sync::Mutex::new(Grid2D::<i32>::new(10, 10)));
    let mut handles = vec![];
    
    // Spawn multiple threads to write to the grid
    for i in 0..10 {
        let grid_clone = Arc::clone(&grid);
        let handle = thread::spawn(move || {
            let mut grid = grid_clone.lock().unwrap();
            grid.set(i, i, i as i32);
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify data
    let final_grid = grid.lock().unwrap();
    for i in 0..10 {
        assert_eq!(final_grid.get(i, i), Some(&(i as i32)));
    }
}
