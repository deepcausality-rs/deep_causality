// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use dcl_data_structures::ring_buffer::prelude::*;

// Helper function to create test dependencies
fn create_test_dependencies(value: Sequence) -> Vec<AtomicSequence> {
    vec![AtomicSequence::from(value)]
}

#[test]
fn test_spinloop_wait_strategy_creation() {
    let _ = SpinLoopWaitStrategy::new();
    assert!(true, "SpinLoopWaitStrategy created successfully");
}

#[test]
fn test_spinloop_wait_strategy_immediate_success() {
    let strategy = SpinLoopWaitStrategy::new();
    let deps = create_test_dependencies(10);
    let alert = AtomicBool::new(false);

    let result = strategy.wait_for(5, &deps, || {
        alert.load(std::sync::atomic::Ordering::Relaxed)
    });
    assert_eq!(result, Some(10));
}

#[test]
fn test_spinloop_wait_strategy_alert() {
    let strategy = SpinLoopWaitStrategy::new();
    let deps = create_test_dependencies(0);
    let alert = AtomicBool::new(true);

    let result = strategy.wait_for(5, &deps, || {
        alert.load(std::sync::atomic::Ordering::Relaxed)
    });
    assert_eq!(result, None);
}

#[test]
fn test_spinloop_wait_strategy_concurrent() {
    let strategy = Arc::new(SpinLoopWaitStrategy::new());
    let deps = Arc::new(create_test_dependencies(0));
    let alert = Arc::new(AtomicBool::new(false));

    let strategy_clone = strategy.clone();
    let deps_clone = deps.clone();
    let alert_clone = alert.clone();

    let handle = thread::spawn(move || {
        strategy_clone.wait_for(10, &deps_clone, || {
            alert_clone.load(std::sync::atomic::Ordering::Relaxed)
        })
    });

    thread::sleep(Duration::from_millis(100));
    deps[0].set(15);
    strategy.signal();

    let result = handle.join().unwrap();
    assert_eq!(result, Some(15));
}

#[test]
fn test_spinloop_wait_strategy_multiple_dependencies() {
    let strategy = SpinLoopWaitStrategy::new();
    let deps = vec![
        AtomicSequence::from(5),
        AtomicSequence::from(10),
        AtomicSequence::from(15),
    ];
    let alert = AtomicBool::new(false);

    let result = strategy.wait_for(3, &deps, || {
        alert.load(std::sync::atomic::Ordering::Relaxed)
    });
    assert_eq!(result, Some(5));
}

#[test]
fn test_spinloop_wait_strategy_stress() {
    let strategy = Arc::new(SpinLoopWaitStrategy::new());
    let deps = Arc::new(create_test_dependencies(0));
    let alert = Arc::new(AtomicBool::new(false));
    let mut handles = vec![];

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

    thread::sleep(Duration::from_millis(50));
    deps[0].set(10);
    strategy.signal();

    for handle in handles {
        assert_eq!(handle.join().unwrap(), Some(10));
    }
}
