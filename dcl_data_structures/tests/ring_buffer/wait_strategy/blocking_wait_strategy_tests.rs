// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use dcl_data_structures::ring_buffer::prelude::*;

// Helper function to create test dependencies
fn create_test_dependencies(value: Sequence) -> Vec<AtomicSequenceOrdered> {
    vec![AtomicSequenceOrdered::from(value)]
}

#[test]
fn test_blocking_wait_strategy_immediate_success() {
    let strategy = BlockingWaitStrategy::new();
    let deps = create_test_dependencies(10);
    let alert = AtomicBool::new(false);

    let result = strategy.wait_for(5, &deps, || {
        alert.load(std::sync::atomic::Ordering::Relaxed)
    });
    assert_eq!(result, Some(10));
}

#[test]
fn test_blocking_wait_strategy_alert() {
    let strategy = BlockingWaitStrategy::new();
    let deps = create_test_dependencies(0);
    let alert = AtomicBool::new(true);

    let result = strategy.wait_for(5, &deps, || {
        alert.load(std::sync::atomic::Ordering::Relaxed)
    });
    assert_eq!(result, None);
}

#[test]
fn test_blocking_wait_strategy_concurrent() {
    let strategy = Arc::new(BlockingWaitStrategy::new());
    let deps = Arc::new(create_test_dependencies(0));
    let alert = Arc::new(AtomicBool::new(false));

    let strategy_clone = strategy.clone();
    let deps_clone = deps.clone();
    let alert_clone = alert.clone();

    // Spawn a thread that will wait for sequence 10
    let handle = thread::spawn(move || {
        strategy_clone.wait_for(10, &deps_clone, || {
            alert_clone.load(std::sync::atomic::Ordering::Relaxed)
        })
    });

    // Sleep briefly to ensure the other thread is waiting
    thread::sleep(Duration::from_millis(100));

    // Update the sequence and signal
    deps[0].set(15);
    strategy.signal();

    // Wait for the result
    let result = handle.join().unwrap();
    assert_eq!(result, Some(15));
}

#[test]
fn test_blocking_wait_strategy_multiple_dependencies() {
    let strategy = BlockingWaitStrategy::new();
    let deps = vec![
        AtomicSequenceOrdered::from(5),
        AtomicSequenceOrdered::from(10),
        AtomicSequenceOrdered::from(15),
    ];
    let alert = AtomicBool::new(false);

    // Test with sequence less than minimum
    let result = strategy.wait_for(3, &deps, || {
        alert.load(std::sync::atomic::Ordering::Relaxed)
    });
    assert_eq!(result, Some(5)); // Should return minimum sequence
}

#[test]
fn test_blocking_wait_strategy_signal_all() {
    let strategy = Arc::new(BlockingWaitStrategy::new());
    let deps = Arc::new(create_test_dependencies(0));
    let alert = Arc::new(AtomicBool::new(false));

    let mut handles = vec![];

    // Spawn fewer threads with smaller sequence numbers
    for i in 1..=2 {
        let strategy = strategy.clone();
        let deps = deps.clone();
        let alert = alert.clone();
        let seq = i * 5;

        handles.push(thread::spawn(move || {
            strategy.wait_for(seq, &deps, || {
                alert.load(std::sync::atomic::Ordering::Relaxed)
            })
        }));
    }

    // Shorter sleep and smaller sequence number
    thread::sleep(Duration::from_millis(50));

    deps[0].set(10);
    strategy.signal();

    for handle in handles {
        assert_eq!(handle.join().unwrap(), Some(10));
    }
}
