use crossbeam_utils::atomic::AtomicCell;
use dcl_data_structures::ring_buffer::prelude::*;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Default)]
struct TestData {
    value: i32,
}

#[allow(dead_code)]
struct TestDataProvider {
    data: Vec<AtomicCell<TestData>>,
}

impl DataProvider<TestData> for TestDataProvider {
    unsafe fn get(&self, sequence: Sequence) -> &TestData {
        let cell = &self.data[sequence as usize % self.data.len()];
        // Safety: AtomicCell guarantees thread-safe access
        std::mem::transmute(cell.as_ptr())
    }

    unsafe fn get_mut(&self, sequence: Sequence) -> &mut TestData {
        let cell = &self.data[sequence as usize % self.data.len()];
        // Safety: AtomicCell guarantees thread-safe access
        std::mem::transmute(cell.as_ptr())
    }

    fn buffer_size(&self) -> usize {
        self.data.len()
    }
}

#[test]
fn test_multi_producer_creation() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let sequencer = MultiProducerSequencer::new(buffer_size, wait_strategy);

    assert_eq!(sequencer.get_cursor().get(), 0);
}

#[test]
fn test_multi_producer_next_sequence() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let sequencer = MultiProducerSequencer::new(buffer_size, wait_strategy);

    let (start, end) = sequencer.next(3);
    assert_eq!(start, 1);
    assert_eq!(end, 3);

    let (start2, end2) = sequencer.next(2);
    assert_eq!(start2, 4);
    assert_eq!(end2, 5);
}

#[test]
fn test_multi_producer_publish() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let sequencer = MultiProducerSequencer::new(buffer_size, wait_strategy);

    let (start, end) = sequencer.next(3);
    sequencer.publish(start, end);
    assert_eq!(sequencer.get_cursor().get(), end);
}

#[test]
fn test_multi_producer_gating_sequence() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let mut sequencer = MultiProducerSequencer::new(buffer_size, wait_strategy);

    let gating_sequence = Arc::new(AtomicSequence::default());
    sequencer.add_gating_sequence(&gating_sequence);

    // The producer should respect the gating sequence
    let (start, end) = sequencer.next(1);
    assert_eq!(start, 1);
    assert_eq!(end, 1);

    // Publish the sequence
    sequencer.publish(start, end);

    // Next request should be able to proceed
    let (start2, end2) = sequencer.next(1);
    assert_eq!(start2, 2);
    assert_eq!(end2, 2);
}

#[test]
fn test_multi_producer_barrier() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let mut sequencer = MultiProducerSequencer::new(buffer_size, wait_strategy);

    let gating_sequences = vec![Arc::new(AtomicSequence::default())];
    let barrier = sequencer.create_barrier(&gating_sequences);

    // Check barrier's sequence is initialized
    assert!(barrier.wait_for(0).is_some());
}
