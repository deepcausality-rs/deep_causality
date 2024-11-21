// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use dcl_data_structures::prelude::{ArrayStorage, VectorStorage, WindowStorage};

#[test]
fn test_vector_storage_capacity_limits() {
    // Test with small capacity
    let mut storage = VectorStorage::<i32>::new(2, 4); // size=2, capacity=8
    assert_eq!(storage.size(), 2);
    assert_eq!(storage.capacity(), 8);

    // Fill to capacity
    for i in 0..8 {
        storage.push(i as i32);
    }
    assert!(storage.filled());
    
    // Test overflow behavior
    storage.push(8);
    assert_eq!(storage.tail(), 9);
    assert_eq!(storage.to_vec(), vec![7, 8]);
}

#[test]
fn test_array_storage_capacity_limits() {
    const SIZE: usize = 2;
    const CAPACITY: usize = 8;
    let mut storage = ArrayStorage::<i32, SIZE, CAPACITY>::new();
    assert_eq!(storage.size(), SIZE);
    assert_eq!(storage.capacity(), CAPACITY);

    // Fill to capacity
    for i in 0..CAPACITY {
        storage.push(i as i32);
    }
    assert!(storage.filled());
    
    // Test overflow behavior
    storage.push(8);
    assert_eq!(storage.tail(), CAPACITY + 1);
    assert_eq!(storage.to_vec(), vec![7, 8]);
}

#[test]
fn test_vector_storage_memory_behavior() {
    let mut storage = VectorStorage::<String>::new(3, 3); // size=3, capacity=9
    
    // Test with heap-allocated data
    storage.push("test1".to_string());
    storage.push("test2".to_string());
    storage.push("test3".to_string());
    
    // Verify data is correctly stored and retrieved
    assert_eq!(storage.get(0), Some(&"test1".to_string()));
    assert_eq!(storage.get(1), Some(&"test2".to_string()));
    assert_eq!(storage.get(2), Some(&"test3".to_string()));
    
    // Test memory cleanup on overflow
    for i in 4..10 {
        storage.push(format!("test{}", i));
    }
    
    // Verify old data is properly cleaned up
    assert_eq!(storage.get(0), Some(&"test7".to_string()));
    assert_eq!(storage.get(1), Some(&"test8".to_string()));
    assert_eq!(storage.get(2), Some(&"test9".to_string()));
}

#[test]
fn test_array_storage_memory_behavior() {
    const SIZE: usize = 3;
    const CAPACITY: usize = 9;
    let mut storage = ArrayStorage::<String, SIZE, CAPACITY>::new();
    
    // Test with heap-allocated data
    storage.push("test1".to_string());
    storage.push("test2".to_string());
    storage.push("test3".to_string());
    
    // Verify data is correctly stored and retrieved
    assert_eq!(storage.get(0), Some(&"test1".to_string()));
    assert_eq!(storage.get(1), Some(&"test2".to_string()));
    assert_eq!(storage.get(2), Some(&"test3".to_string()));
    
    // Test memory cleanup on overflow
    for i in 4..10 {
        storage.push(format!("test{}", i));
    }
    
    // Verify old data is properly cleaned up
    assert_eq!(storage.get(0), Some(&"test7".to_string()));
    assert_eq!(storage.get(1), Some(&"test8".to_string()));
    assert_eq!(storage.get(2), Some(&"test9".to_string()));
}

#[test]
fn test_vector_storage_concurrent_access() {
    use std::sync::Arc;
    use std::thread;
    
    let storage = Arc::new(std::sync::Mutex::new(VectorStorage::<i32>::new(5, 2)));
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
fn test_edge_cases() {
    // Test with zero size
    let storage = VectorStorage::<i32>::new(0, 1);
    assert!(storage.empty());
    assert_eq!(storage.size(), 0);
    
    // Test with equal size and capacity multiplier
    let mut storage = VectorStorage::<i32>::new(2, 1);
    assert_eq!(storage.capacity(), 2);
    
    // Test pushing more elements than capacity
    for i in 0..5 {
        storage.push(i);
    }
    assert_eq!(storage.to_vec(), vec![3, 4]);
    
    // Test with maximum values
    let large_size = 1000000;
    let storage = VectorStorage::<i32>::new(large_size, 2);
    assert_eq!(storage.size(), large_size);
    assert_eq!(storage.capacity(), large_size * 2);
}

#[test]
fn test_performance_comparison() {
    use std::time::Instant;
    
    const SIZE: usize = 1000;
    const MULTIPLIER: usize = 10;
    
    // Vector Storage Performance
    let start = Instant::now();
    let mut vec_storage = VectorStorage::<i32>::new(SIZE, MULTIPLIER);
    for i in 0..SIZE*MULTIPLIER {
        vec_storage.push(i as i32);
    }
    let vec_duration = start.elapsed();
    
    // Array Storage Performance
    let start = Instant::now();
    let mut arr_storage = ArrayStorage::<i32, SIZE, { SIZE * MULTIPLIER }>::new();
    for i in 0..SIZE*MULTIPLIER {
        arr_storage.push(i as i32);
    }
    let arr_duration = start.elapsed();
    
    println!("Vector Storage Duration: {:?}", vec_duration);
    println!("Array Storage Duration: {:?}", arr_duration);
    
    // Verify both storages have the same content
    assert_eq!(vec_storage.tail(), arr_storage.tail());
    assert_eq!(vec_storage.to_vec(), arr_storage.to_vec());
}
