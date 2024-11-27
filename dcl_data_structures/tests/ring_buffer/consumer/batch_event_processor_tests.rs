// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use dcl_data_structures::ring_buffer::prelude::*;

// Test helper structs
#[derive(Debug, Clone)]
struct TestData {
    value: usize,
}

struct TestEventHandler {
    counter: Arc<AtomicUsize>,
    processed: Arc<AtomicBool>,
}

impl EventHandler<TestData> for TestEventHandler {
    fn handle_event(&self, _event: &TestData, _sequence: Sequence, is_end_of_batch: bool) {
        self.counter.fetch_add(1, Ordering::SeqCst);
        if is_end_of_batch {
            self.processed.store(true, Ordering::SeqCst);
        }
    }
}

struct TestEventHandlerMut {
    counter: Arc<AtomicUsize>,
    processed: Arc<AtomicBool>,
}

impl EventHandlerMut<TestData> for TestEventHandlerMut {
    fn handle_event(&mut self, event: &mut TestData, _sequence: Sequence, is_end_of_batch: bool) {
        event.value += 1;
        self.counter.fetch_add(1, Ordering::SeqCst);
        if is_end_of_batch {
            self.processed.store(true, Ordering::SeqCst);
        }
    }
}

struct TestDataProvider {
    data: Vec<TestData>,
}

impl DataProvider<TestData> for TestDataProvider {
    fn buffer_size(&self) -> usize {
        self.data.len()
    }

    unsafe fn get_mut(&self, sequence: Sequence) -> &mut TestData {
        if sequence as usize >= self.data.len() {
            panic!(
                "Sequence {} is out of bounds (len: {})",
                sequence,
                self.data.len()
            );
        }
        let ptr = self.data.as_ptr().add(sequence as usize) as *mut TestData;
        &mut *ptr
    }

    unsafe fn get(&self, sequence: Sequence) -> &TestData {
        if sequence as usize >= self.data.len() {
            panic!(
                "Sequence {} is out of bounds (len: {})",
                sequence,
                self.data.len()
            );
        }
        &self.data[sequence as usize]
    }
}

#[allow(dead_code)]
struct TestBarrier {
    cursor: Arc<AtomicSequenceOrdered>,
    wait_strategy: Arc<SpinLoopWaitStrategy>,
    dependent_sequences: Vec<Arc<AtomicSequenceOrdered>>,
    processed_count: Arc<AtomicUsize>,
    max_sequence: Arc<AtomicSequenceOrdered>,
}

impl SequenceBarrier for TestBarrier {
    fn wait_for(&self, sequence: u64) -> Option<u64> {
        let max_seq = self.max_sequence.get();
        if sequence > max_seq {
            None
        } else {
            Some(sequence)
        }
    }

    fn signal(&self) {
        // For testing, we don't need to do anything here
    }
}

#[test]
fn test_immutable_processor_creation() {
    let counter = Arc::new(AtomicUsize::new(0));
    let processed = Arc::new(AtomicBool::new(false));
    let handler = TestEventHandler {
        counter: counter.clone(),
        processed: processed.clone(),
    };

    let processor = BatchEventProcessor::create(handler);
    assert_eq!(processor.get_cursor().get(), 0);
}

#[test]
fn test_mutable_processor_creation() {
    let counter = Arc::new(AtomicUsize::new(0));
    let processed = Arc::new(AtomicBool::new(false));
    let handler = TestEventHandlerMut {
        counter: counter.clone(),
        processed: processed.clone(),
    };

    let processor = BatchEventProcessor::create_mut(handler);
    assert_eq!(processor.get_cursor().get(), 0);
}

#[test]
fn test_immutable_processor_event_handling() {
    let counter = Arc::new(AtomicUsize::new(0));
    let processed = Arc::new(AtomicBool::new(false));
    let handler = TestEventHandler {
        counter: counter.clone(),
        processed: processed.clone(),
    };

    let data = vec![
        TestData { value: 1 },
        TestData { value: 2 },
        TestData { value: 3 },
    ];
    let data_provider = Arc::new(TestDataProvider { data });

    let wait_strategy = Arc::new(SpinLoopWaitStrategy::new());
    let max_sequence = Arc::new(AtomicSequenceOrdered::default());
    max_sequence.set(2); // We have 3 elements (0, 1, 2)

    let barrier = TestBarrier {
        cursor: Arc::new(AtomicSequenceOrdered::default()),
        wait_strategy,
        dependent_sequences: vec![],
        processed_count: Arc::new(AtomicUsize::new(0)),
        max_sequence,
    };

    let processor = BatchEventProcessor::create(handler);
    let runnable = processor.prepare(barrier, data_provider);

    // Run in a separate thread
    let handle = thread::spawn(move || {
        runnable.run();
    });

    // Wait a bit for processing
    thread::sleep(Duration::from_millis(100));

    assert_eq!(counter.load(Ordering::SeqCst), 2);
    assert!(processed.load(Ordering::SeqCst));

    handle.join().unwrap();
}

#[test]
fn test_mutable_processor_event_handling() {
    let counter = Arc::new(AtomicUsize::new(0));
    let processed = Arc::new(AtomicBool::new(false));
    let handler = TestEventHandlerMut {
        counter: counter.clone(),
        processed: processed.clone(),
    };

    let data = vec![
        TestData { value: 1 },
        TestData { value: 2 },
        TestData { value: 3 },
    ];
    let data_provider = Arc::new(TestDataProvider { data });

    let wait_strategy = Arc::new(SpinLoopWaitStrategy::new());
    let max_sequence = Arc::new(AtomicSequenceOrdered::default());
    max_sequence.set(2); // We have 3 elements (0, 1, 2)

    let barrier = TestBarrier {
        cursor: Arc::new(AtomicSequenceOrdered::default()),
        wait_strategy,
        dependent_sequences: vec![],
        processed_count: Arc::new(AtomicUsize::new(0)),
        max_sequence,
    };

    let processor = BatchEventProcessor::create_mut(handler);
    let runnable = processor.prepare(barrier, data_provider.clone());

    // Run in a separate thread
    let handle = thread::spawn(move || {
        runnable.run();
    });

    // Wait a bit for processing
    thread::sleep(Duration::from_millis(100));

    assert_eq!(counter.load(Ordering::SeqCst), 2);
    assert!(processed.load(Ordering::SeqCst));

    handle.join().unwrap();
}

#[test]
fn test_processor_with_dependencies() {
    let counter = Arc::new(AtomicUsize::new(0));
    let processed = Arc::new(AtomicBool::new(false));
    let handler = TestEventHandler {
        counter: counter.clone(),
        processed: processed.clone(),
    };

    let data = vec![
        TestData { value: 1 },
        TestData { value: 2 },
        TestData { value: 3 },
    ];
    let data_provider = Arc::new(TestDataProvider { data });

    let wait_strategy = Arc::new(SpinLoopWaitStrategy::new());
    let dependent_sequence = Arc::new(AtomicSequenceOrdered::default());
    let max_sequence = Arc::new(AtomicSequenceOrdered::default());
    max_sequence.set(2); // We have 3 elements (0, 1, 2)

    let barrier = TestBarrier {
        cursor: Arc::new(AtomicSequenceOrdered::default()),
        wait_strategy,
        dependent_sequences: vec![dependent_sequence.clone()],
        processed_count: Arc::new(AtomicUsize::new(0)),
        max_sequence,
    };

    let processor = BatchEventProcessor::create(handler);
    let runnable = processor.prepare(barrier, data_provider);

    // Run in a separate thread
    let handle = thread::spawn(move || {
        runnable.run();
    });

    // Advance the dependent sequence to allow processing
    dependent_sequence.set(2);

    // Wait a bit for processing
    thread::sleep(Duration::from_millis(100));

    assert_eq!(counter.load(Ordering::SeqCst), 2);
    assert!(processed.load(Ordering::SeqCst));

    handle.join().unwrap();
}

#[test]
fn test_processor_cursor_progression() {
    let counter = Arc::new(AtomicUsize::new(0));
    let processed = Arc::new(AtomicBool::new(false));
    let handler = TestEventHandler {
        counter: counter.clone(),
        processed: processed.clone(),
    };

    let data = vec![
        TestData { value: 1 },
        TestData { value: 2 },
        TestData { value: 3 },
    ];
    let data_provider = Arc::new(TestDataProvider { data });

    let wait_strategy = Arc::new(SpinLoopWaitStrategy::new());
    let cursor = Arc::new(AtomicSequenceOrdered::default());
    let max_sequence = Arc::new(AtomicSequenceOrdered::default());
    max_sequence.set(2); // We have 3 elements (0, 1, 2)

    let barrier = TestBarrier {
        cursor: cursor.clone(),
        wait_strategy,
        dependent_sequences: vec![],
        processed_count: Arc::new(AtomicUsize::new(0)),
        max_sequence,
    };

    let processor = BatchEventProcessor::create(handler);
    let processor_cursor = processor.get_cursor();
    let runnable = processor.prepare(barrier, data_provider);

    // Run in a separate thread
    let handle = thread::spawn(move || {
        runnable.run();
    });

    // Wait a bit and check cursor progression
    thread::sleep(Duration::from_millis(100));
    assert!(processor_cursor.get() > 0);

    thread::sleep(Duration::from_millis(100));
    assert_eq!(processor_cursor.get(), 2); // Should be at the last sequence

    handle.join().unwrap();
}
