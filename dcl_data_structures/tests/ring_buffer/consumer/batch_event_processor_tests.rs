// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use dcl_data_structures::ring_buffer::prelude::*;

// Test helper structs
#[allow(dead_code)]
struct TestData {
    value: usize,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
struct TestDataProvider {
    data: Vec<TestData>,
}

impl DataProvider<TestData> for TestDataProvider {
    fn buffer_size(&self) -> usize {
        self.data.len()
    }

    unsafe fn get_mut(&self, sequence: Sequence) -> &mut TestData {
        // Safe because we know the data is mutable and the sequence is valid
        let ptr = self.data.as_ptr().add(sequence as usize) as *mut TestData;
        &mut *ptr
    }

    unsafe fn get(&self, sequence: Sequence) -> &TestData {
        &self.data[sequence as usize]
    }
}

#[test]
fn test_batch_processor_creation() {
    let counter = Arc::new(AtomicUsize::new(0));
    let processed = Arc::new(AtomicBool::new(false));
    let handler = TestEventHandler {
        counter: counter.clone(),
        processed: processed.clone(),
    };

    let processor = BatchEventProcessor::create(handler);
    assert_eq!(processor.get_cursor().get(), 0);
}

// Add more tests later on
