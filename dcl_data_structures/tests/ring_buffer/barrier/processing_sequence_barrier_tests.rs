// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use dcl_data_structures::ring_buffer::prelude::*;

// Mock wait strategy for testing
struct TestWaitStrategy {
    counter: AtomicSequence,
}

impl TestWaitStrategy {
    fn new() -> Self {
        Self {
            counter: AtomicSequence::default(),
        }
    }
}

impl WaitStrategy for TestWaitStrategy {
    fn new() -> Self {
        Self::new()
    }

    fn wait_for<F: Fn() -> bool, S: std::borrow::Borrow<AtomicSequence>>(
        &self,
        sequence: Sequence,
        _dependencies: &[S],
        alert_condition: F,
    ) -> Option<Sequence> {
        // Check alert condition first
        if alert_condition() {
            return None;
        }
        self.counter.set(sequence);
        Some(sequence)
    }

    fn signal(&self) {
        let current = self.counter.get();
        self.counter.set(current + 1);
    }
}

#[test]
fn test_barrier_creation() {
    let wait_strategy = Arc::new(TestWaitStrategy::new());
    let gating_sequences = vec![Arc::new(AtomicSequence::default())];
    let is_alerted = Arc::new(AtomicBool::new(false));

    let _barrier = ProcessingSequenceBarrier::new(wait_strategy, gating_sequences, is_alerted);
}

#[test]
fn test_barrier_wait_for() {
    let wait_strategy = Arc::new(TestWaitStrategy::new());
    let gating_sequences = vec![Arc::new(AtomicSequence::default())];
    let is_alerted = Arc::new(AtomicBool::new(false));

    let barrier = ProcessingSequenceBarrier::new(wait_strategy, gating_sequences, is_alerted);
    let result = barrier.wait_for(42);
    assert_eq!(result, Some(42));
}

#[test]
fn test_barrier_signal() {
    let wait_strategy = Arc::new(TestWaitStrategy::new());
    let gating_sequences = vec![Arc::new(AtomicSequence::default())];
    let is_alerted = Arc::new(AtomicBool::new(false));

    let barrier =
        ProcessingSequenceBarrier::new(Arc::clone(&wait_strategy), gating_sequences, is_alerted);

    barrier.signal();
    assert_eq!(wait_strategy.counter.get(), 1);
}

#[test]
fn test_barrier_with_alert_condition() {
    let wait_strategy = Arc::new(TestWaitStrategy::new()); // Use TestWaitStrategy instead
    let gating_sequences = vec![Arc::new(AtomicSequence::default())];
    let is_alerted = Arc::new(AtomicBool::new(false));

    let barrier =
        ProcessingSequenceBarrier::new(wait_strategy, gating_sequences, is_alerted.clone());

    // Set the alert condition
    is_alerted.store(true, Ordering::Relaxed);

    // This should return None because the barrier is alerted
    let result = barrier.wait_for(100);
    assert_eq!(result, None);
}

#[test]
fn test_barrier_multiple_gating_sequences() {
    let wait_strategy = Arc::new(TestWaitStrategy::new());
    let gating_sequences = vec![
        Arc::new(AtomicSequence::default()),
        Arc::new(AtomicSequence::default()),
        Arc::new(AtomicSequence::default()),
    ];
    let is_alerted = Arc::new(AtomicBool::new(false));

    let barrier = ProcessingSequenceBarrier::new(wait_strategy, gating_sequences, is_alerted);
    let result = barrier.wait_for(42);
    assert_eq!(result, Some(42));
}

#[test]
fn test_barrier_concurrent_access() {
    let wait_strategy = Arc::new(TestWaitStrategy::new());
    let gating_sequences = vec![Arc::new(AtomicSequence::default())];
    let is_alerted = Arc::new(AtomicBool::new(false));

    let barrier = Arc::new(ProcessingSequenceBarrier::new(
        wait_strategy,
        gating_sequences,
        is_alerted,
    ));

    let mut handles = vec![];

    // Spawn multiple threads that will wait for different sequences
    for i in 0..5 {
        let barrier_clone = Arc::clone(&barrier);
        let handle = thread::spawn(move || {
            let sequence = i * 10;
            let result = barrier_clone.wait_for(sequence as Sequence);
            assert_eq!(result, Some(sequence as Sequence));
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}
