/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use dcl_data_structures::prelude::{UnsafeVectorStorage, WindowStorage};

#[test]
fn test_vector_storage_capacity_limits() {
    // Test with small capacity
    let mut storage = UnsafeVectorStorage::<i32>::new(2, 4); // size=2, capacity=8
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

#[test]
fn test_vector_storage_memory_behavior() {
    let mut storage = UnsafeVectorStorage::<i32>::new(3, 3); // size=3, capacity=9

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

#[test]
fn test_vector_storage_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let storage = Arc::new(std::sync::Mutex::new(UnsafeVectorStorage::<i32>::new(5, 2)));
    let mut handles = vec![];

    // Spawn multiple threads to push data
    for i in 0..5 {
        let storage_clone = Arc::clone(&storage);
        let handle = thread::spawn(move || {
            let mut storage = storage_clone.lock().unwrap();
            storage.push(i);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify data
    let final_storage = storage.lock().unwrap();
    assert_eq!(final_storage.tail(), 5);
    assert!(final_storage.filled());
}

#[test]
fn test_vector_storage_edge_cases() {
    // Test with minimum size
    let mut storage = UnsafeVectorStorage::<i32>::new(1, 2);
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
    let empty_storage = UnsafeVectorStorage::<i32>::new(1, 2);
    assert!(empty_storage.first().is_err());
    assert!(empty_storage.last().is_err());
}
