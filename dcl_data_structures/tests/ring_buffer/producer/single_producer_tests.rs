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

impl TestDataProvider {
    fn new(size: usize) -> Self {
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(AtomicCell::new(TestData::default()));
        }
        TestDataProvider { data }
    }
}

impl DataProvider<TestData> for TestDataProvider {
    unsafe fn get(&self, sequence: Sequence) -> &TestData {
        let cell = &self.data[sequence as usize % self.data.len()];
        // Safety: AtomicCell guarantees thread-safe access
        &*cell.as_ptr()
    }

    unsafe fn get_mut(&self, sequence: Sequence) -> &mut TestData {
        let cell = &self.data[sequence as usize % self.data.len()];
        // Safety: AtomicCell guarantees thread-safe access
        &mut *cell.as_ptr()
    }

    fn buffer_size(&self) -> usize {
        self.data.len()
    }
}

#[test]
fn test_single_producer_creation() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let sequencer = SingleProducerSequencer::new(buffer_size, wait_strategy);

    assert_eq!(sequencer.get_cursor().get(), 0);
}

#[test]
fn test_single_producer_next_sequence() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let sequencer = SingleProducerSequencer::new(buffer_size, wait_strategy);

    let (start, end) = sequencer.next(3);
    assert_eq!(start, 0);
    assert_eq!(end, 2);

    let (start2, end2) = sequencer.next(2);
    assert_eq!(start2, 3);
    assert_eq!(end2, 4);
}

#[test]
fn test_single_producer_publish() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let sequencer = SingleProducerSequencer::new(buffer_size, wait_strategy);

    let (start, end) = sequencer.next(3);
    sequencer.publish(start, end);
    assert_eq!(sequencer.get_cursor().get(), end);
}

#[test]
fn test_single_producer_write() {
    let buffer_size = 8;
    let data_provider = Arc::new(TestDataProvider::new(buffer_size));
    let wait_strategy = BlockingWaitStrategy::new();
    let sequencer = SingleProducerSequencer::new(buffer_size, wait_strategy);
    let producer = Producer::new(data_provider.clone(), sequencer);

    let items = vec![1, 2, 3];
    producer.write(items, |slot, _, &value| {
        slot.value = value;
    });

    // Verify the written values
    unsafe {
        assert_eq!(data_provider.get(0).value, 1);
        assert_eq!(data_provider.get(1).value, 2);
        assert_eq!(data_provider.get(2).value, 3);
    }
}

#[test]
fn test_single_producer_gating_sequence() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let mut sequencer = SingleProducerSequencer::new(buffer_size, wait_strategy);

    let gating_sequence = Arc::new(AtomicSequence::default());
    sequencer.add_gating_sequence(&gating_sequence);

    // The producer should respect the gating sequence
    let (start, end) = sequencer.next(buffer_size);
    assert_eq!(start, 0);
    assert_eq!(end, buffer_size as u64 - 1);

    // Next request should wait for gating sequence to advance
    gating_sequence.set(buffer_size as u64 - 1);
    let (start2, end2) = sequencer.next(1);
    assert_eq!(start2, buffer_size as u64);
    assert_eq!(end2, buffer_size as u64);
}

#[test]
fn test_single_producer_barrier() {
    let buffer_size = 8;
    let wait_strategy = BlockingWaitStrategy::new();
    let mut sequencer = SingleProducerSequencer::new(buffer_size, wait_strategy);

    let gating_sequences = vec![Arc::new(AtomicSequence::default())];
    let barrier = sequencer.create_barrier(&gating_sequences);

    assert!(barrier.wait_for(0).is_some());
}
