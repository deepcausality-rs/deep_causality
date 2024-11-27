// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::sync::Arc;
use std::thread;

use dcl_data_structures::ring_buffer::prelude::*;

#[test]
fn test_atomic_sequence_default() {
    let seq = AtomicSequenceRelaxed::default();
    assert_eq!(seq.get(), 0);
}

#[test]
fn test_atomic_sequence_from() {
    let value: Sequence = 42;
    let seq = AtomicSequenceRelaxed::from(value);
    assert_eq!(seq.get(), value as u64);
}

#[test]
fn test_atomic_sequence_into() {
    let value: Sequence = 42;
    let seq = AtomicSequenceRelaxed::from(value);
    let result: Sequence = seq.into();
    assert_eq!(result, value);
}

#[test]
fn test_atomic_sequence_get_set() {
    let seq = AtomicSequenceRelaxed::default();
    seq.set(100);
    assert_eq!(seq.get(), 100);
}

#[test]
fn test_atomic_sequence_increment() {
    let seq = AtomicSequenceRelaxed::default();

    // First increment should return 0 (previous value) and set value to 1
    assert_eq!(seq.increment(), 0);
    assert_eq!(seq.get(), 1);

    // Second increment should return 1 and set value to 2
    assert_eq!(seq.increment(), 1);
    assert_eq!(seq.get(), 2);

    // Test multiple increments in sequence
    for i in 2..10 {
        assert_eq!(seq.increment(), i);
        assert_eq!(seq.get(), i + 1);
    }
}

#[test]
fn test_atomic_sequence_compare_exchange_success() {
    let seq = AtomicSequenceRelaxed::default();
    assert!(seq.compare_and_swap(0, 1));
    assert_eq!(seq.get(), 1);
}

#[test]
fn test_atomic_sequence_compare_exchange_failure() {
    let seq = AtomicSequenceRelaxed::default();
    seq.set(5);
    assert!(!seq.compare_and_swap(0, 1));
    assert_eq!(seq.get(), 5);
}

#[test]
fn test_atomic_sequence_clone() {
    let seq = AtomicSequenceRelaxed::from(42);
    let cloned = seq.clone();
    assert_eq!(seq.get(), cloned.get());

    // Verify that changes to original don't affect clone
    seq.set(100);
    assert_eq!(seq.get(), 100);
    assert_eq!(cloned.get(), 42);
}

#[test]
fn test_atomic_sequence_thread_safety() {
    let seq = Arc::new(AtomicSequenceRelaxed::default());
    let mut handles = vec![];

    // Spawn 10 threads that increment the sequence
    for _ in 0..10 {
        let seq_clone = Arc::clone(&seq);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                seq_clone.increment();
            }
        }));
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Each thread increments 1000 times, so final value should be 10000
    assert_eq!(seq.get(), 10000);
}

#[test]
fn test_atomic_sequence_concurrent_reads() {
    let seq = Arc::new(AtomicSequenceRelaxed::from(42));
    let mut handles = vec![];

    // Spawn multiple reader threads
    for _ in 0..5 {
        let seq_clone = Arc::clone(&seq);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                assert_eq!(seq_clone.get(), 42);
            }
        }));
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_atomic_sequence_multiple_modifications() {
    let seq = AtomicSequenceRelaxed::default();

    // Test a series of operations
    seq.set(1);
    assert_eq!(seq.get(), 1);

    seq.increment();
    assert_eq!(seq.get(), 2);

    assert!(seq.compare_and_swap(2, 3));
    assert_eq!(seq.get(), 3);

    assert!(!seq.compare_and_swap(2, 4)); // Should fail
    assert_eq!(seq.get(), 3); // Value should remain unchanged
}

#[test]
fn test_atomic_sequence_concurrent_increment() {
    let seq = Arc::new(AtomicSequenceRelaxed::default());
    let mut handles = vec![];
    let num_threads = 10;
    let increments_per_thread = 1000;

    // Spawn multiple threads that increment the sequence
    for _ in 0..num_threads {
        let seq_clone = Arc::clone(&seq);
        handles.push(thread::spawn(move || {
            for _ in 0..increments_per_thread {
                seq_clone.increment();
            }
        }));
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}
