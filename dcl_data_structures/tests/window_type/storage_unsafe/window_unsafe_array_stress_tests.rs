/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{UnsafeArrayStorage, WindowStorage};

#[cfg(feature = "unsafe")]
#[test]
fn test_vector_storage_capacity_limits() {
    const SIZE: usize = 2;
    const CAPACITY: usize = 8;
    // Test with small capacity
    let mut storage = UnsafeArrayStorage::<i32, SIZE, CAPACITY>::new(); // size=2, capacity=8
    assert_eq!(storage.size(), 2);

    // Fill to capacity
    for i in 0..4 {
        storage.push(i);
    }
    assert!(storage.filled());

    // Test overflow behavior
    storage.push(4);
    assert_eq!(storage.tail(), 5);
    assert_eq!(storage.vec().unwrap(), vec![3, 4]);
}

#[cfg(feature = "unsafe")]
#[test]
fn test_vector_storage_memory_behavior() {
    const SIZE: usize = 3;
    const CAPACITY: usize = 9;
    // Test with small capacity
    let mut storage = UnsafeArrayStorage::<i32, SIZE, CAPACITY>::new(); // size=3, capacity=9

    // Test with stack-allocated data
    storage.push(1);
    storage.push(2);
    storage.push(3);

    assert!(storage.filled());
    assert_eq!(storage.vec().unwrap(), vec![1, 2, 3]);

    // Test overflow behavior
    storage.push(4);
    assert_eq!(storage.vec().unwrap(), vec![2, 3, 4]);
}

#[cfg(feature = "unsafe")]
#[test]
fn test_array_storage_memory_behavior() {
    const SIZE: usize = 3;
    const CAPACITY: usize = 9;
    let mut storage = UnsafeArrayStorage::<i32, SIZE, CAPACITY>::new();

    // Test with stack-allocated data
    storage.push(1);
    storage.push(2);
    storage.push(3);

    assert!(storage.filled());
    assert_eq!(storage.vec().unwrap(), vec![1, 2, 3]);

    // Test overflow behavior
    storage.push(4);
    assert_eq!(storage.vec().unwrap(), vec![2, 3, 4]);
}

#[cfg(feature = "unsafe")]
#[test]
fn test_edge_cases() {
    const SIZE: usize = 1;
    const CAPACITY: usize = 2;
    let mut storage = UnsafeArrayStorage::<i32, SIZE, CAPACITY>::new();
    assert_eq!(storage.size(), 1);

    // Test single element behavior
    storage.push(1);
    assert!(storage.filled());
    assert_eq!(storage.first().unwrap(), 1);
    assert_eq!(storage.last().unwrap(), 1);

    // Test overflow with size 1
    storage.push(2);
    assert_eq!(storage.first(), Ok(2));
    assert_eq!(storage.last(), Ok(2));

    // Test with zero pushes
    let empty_storage = UnsafeArrayStorage::<i32, SIZE, CAPACITY>::new();
    assert!(empty_storage.first().is_err());
    assert!(empty_storage.last().is_err());
}
