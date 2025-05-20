// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dcl_data_structures::ring_buffer::prelude::*;

// A test runnable that increments a counter
struct TestRunnable {
    counter: Arc<AtomicUsize>,
    sleep_duration: Option<Duration>,
    finished: Arc<AtomicBool>,
}

impl TestRunnable {
    fn new(
        counter: Arc<AtomicUsize>,
        sleep_duration: Option<Duration>,
        finished: Arc<AtomicBool>,
    ) -> Self {
        Self {
            counter,
            sleep_duration,
            finished,
        }
    }
}

impl Runnable for TestRunnable {
    fn run(self: Box<Self>) {
        if let Some(duration) = self.sleep_duration {
            std::thread::sleep(duration);
        }
        self.counter.fetch_add(1, Ordering::SeqCst);
        self.finished.store(true, Ordering::SeqCst);
    }
}

#[test]
fn test_executor_single_task() {
    let counter = Arc::new(AtomicUsize::new(0));
    let finished = Arc::new(AtomicBool::new(false));
    let runnable: Box<dyn Runnable> =
        Box::new(TestRunnable::new(counter.clone(), None, finished.clone()));
    let runnables = vec![runnable];

    let executor = ThreadedExecutor::with_runnables(runnables);
    let handle = executor.spawn();
    handle.join();

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert!(finished.load(Ordering::SeqCst));
}

#[test]
fn test_executor_multiple_tasks() {
    let counter = Arc::new(AtomicUsize::new(0));
    let finished1 = Arc::new(AtomicBool::new(false));
    let finished2 = Arc::new(AtomicBool::new(false));
    let finished3 = Arc::new(AtomicBool::new(false));

    let runnables: Vec<Box<dyn Runnable>> = vec![
        Box::new(TestRunnable::new(counter.clone(), None, finished1.clone())),
        Box::new(TestRunnable::new(counter.clone(), None, finished2.clone())),
        Box::new(TestRunnable::new(counter.clone(), None, finished3.clone())),
    ];

    let executor = ThreadedExecutor::with_runnables(runnables);
    let handle = executor.spawn();
    handle.join();

    assert_eq!(counter.load(Ordering::SeqCst), 3);
    assert!(finished1.load(Ordering::SeqCst));
    assert!(finished2.load(Ordering::SeqCst));
    assert!(finished3.load(Ordering::SeqCst));
}

#[test]
fn test_executor_concurrent_execution() {
    let counter = Arc::new(AtomicUsize::new(0));
    let finished1 = Arc::new(AtomicBool::new(false));
    let finished2 = Arc::new(AtomicBool::new(false));

    let runnables: Vec<Box<dyn Runnable>> = vec![
        Box::new(TestRunnable::new(
            counter.clone(),
            Some(Duration::from_millis(100)),
            finished1.clone(),
        )),
        Box::new(TestRunnable::new(
            counter.clone(),
            Some(Duration::from_millis(50)),
            finished2.clone(),
        )),
    ];

    let start = std::time::Instant::now();
    let executor = ThreadedExecutor::with_runnables(runnables);
    let handle = executor.spawn();
    handle.join();
    let elapsed = start.elapsed();

    assert_eq!(counter.load(Ordering::SeqCst), 2);
    assert!(finished1.load(Ordering::SeqCst));
    assert!(finished2.load(Ordering::SeqCst));
    // Tasks should complete in parallel, so total time should be closer to the longest task
    assert!(elapsed < Duration::from_millis(150));
}

#[test]
fn test_executor_handle_drop() {
    let counter = Arc::new(AtomicUsize::new(0));
    let finished = Arc::new(AtomicBool::new(false));
    let runnable: Box<dyn Runnable> = Box::new(TestRunnable::new(
        counter.clone(),
        Some(Duration::from_millis(50)),
        finished.clone(),
    ));
    let runnables = vec![runnable];

    let executor = ThreadedExecutor::with_runnables(runnables);
    let handle = executor.spawn();
    drop(handle); // This should wait for the task to complete

    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert!(finished.load(Ordering::SeqCst));
}
