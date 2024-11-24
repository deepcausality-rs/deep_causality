//! Multi-producer implementation for concurrent event production.
//!
//! This module provides a thread-safe sequencer that allows multiple threads to
//! concurrently produce events into the ring buffer. It uses atomic operations
//! and a bitmap to track sequence availability and ensure proper ordering.
//!
//! # Example
//!
//! ```
//! use dcl_data_structures::ring_buffer::prelude::*;
//! use std::sync::Arc;
//! use crossbeam_utils::atomic::AtomicCell;
//! use std::thread;
//!
//! // Create a data provider for our events
//! #[derive(Clone)]
//! struct MyData(u64);
//! struct MyDataProvider(Vec<AtomicCell<MyData>>);
//!
//! impl DataProvider<MyData> for MyDataProvider {
//!     unsafe fn get(&self, sequence: Sequence) -> &MyData {
//!         let cell = &self.0[sequence as usize % self.0.len()];
//!         std::mem::transmute(cell as *const _)
//!     }
//!     unsafe fn get_mut(&self, sequence: Sequence) -> &mut MyData {
//!         let cell = &self.0[sequence as usize % self.0.len()];
//!         std::mem::transmute(cell as *const _)
//!     }
//!     fn buffer_size(&self) -> usize {
//!         self.0.len()
//!     }
//! }
//!
//! // Create a multi producer that can be shared between threads
//! let buffer_size = 8;
//! let data_provider = Arc::new(MyDataProvider(
//!     (0..buffer_size).map(|_| AtomicCell::new(MyData(0))).collect()
//! ));
//! let wait_strategy = BlockingWaitStrategy::new();
//! let mut sequencer = MultiProducerSequencer::new(buffer_size, wait_strategy);
//!
//! // Create multiple producers in different threads
//! let mut handles = vec![];
//! for i in 0..2 {
//!     let provider = data_provider.clone();
//!     let producer = Producer::new(provider, sequencer.clone());
//!     handles.push(thread::spawn(move || {
//!         producer.write(vec![MyData(i as u64)], |slot, _, item| {
//!             slot.0 = item.0;
//!         });
//!     }));
//! }
//!
//! // Wait for all producers to finish
//! for handle in handles {
//!     handle.join().unwrap();
//! }
//! ```

use crate::ring_buffer::prelude::*;
use std::num::NonZeroUsize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// A sequencer that supports multiple concurrent producers.
///
/// This sequencer uses atomic operations and a bitmap to track sequence availability,
/// ensuring that multiple threads can safely produce events concurrently.
///
/// # Type Parameters
///
/// * `W` - The wait strategy used for coordinating between producers and consumers
pub struct MultiProducerSequencer<W: WaitStrategy> {
    /// The current cursor position in the ring buffer
    cursor: Arc<AtomicSequence>,
    /// The strategy used for waiting when the buffer is full
    wait_strategy: Arc<W>,
    /// Sequences that this producer must wait for before overwriting slots
    gating_sequences: Vec<Arc<AtomicSequence>>,
    /// Size of the ring buffer
    buffer_size: usize,
    /// Tracks the highest claimed sequence
    high_watermark: AtomicSequence,
    /// Bitmap tracking which sequences are ready for publishing
    ready_sequences: BitMap,
    /// Flag indicating if the sequencer has been drained
    is_done: Arc<AtomicBool>,
    low_watermark: AtomicSequence,
}

/// Manual implementation of Clone for MultiProducerSequencer
impl<W: WaitStrategy> Clone for MultiProducerSequencer<W> {
    fn clone(&self) -> Self {
        Self {
            cursor: self.cursor.clone(),
            wait_strategy: self.wait_strategy.clone(),
            gating_sequences: self.gating_sequences.clone(),
            buffer_size: self.buffer_size,
            high_watermark: AtomicSequence::default(),
            ready_sequences: BitMap::new(NonZeroUsize::try_from(self.buffer_size).unwrap()),
            is_done: self.is_done.clone(),
            low_watermark: AtomicSequence::default(),
        }
    }
}

impl<W: WaitStrategy> MultiProducerSequencer<W> {
    /// Creates a new multi-producer sequencer.
    ///
    /// # Arguments
    ///
    /// * `buffer_size` - The size of the ring buffer
    /// * `wait_strategy` - The strategy to use when waiting for available slots
    pub fn new(buffer_size: usize, wait_strategy: W) -> Self {
        MultiProducerSequencer {
            cursor: Arc::new(AtomicSequence::default()),
            wait_strategy: Arc::new(wait_strategy),
            gating_sequences: Vec::new(),
            buffer_size,
            high_watermark: AtomicSequence::default(),
            ready_sequences: BitMap::new(NonZeroUsize::try_from(buffer_size).unwrap()),
            is_done: Default::default(),
            low_watermark: AtomicSequence::default(),
        }
    }

    /// Checks if there is capacity to claim the requested number of sequences.
    ///
    /// # Arguments
    ///
    /// * `high_watermark` - The current high watermark sequence
    /// * `count` - Number of sequences to check capacity for
    ///
    /// # Returns
    ///
    /// `true` if there is enough space in the buffer, `false` otherwise
    fn has_capacity(&self, high_watermark: Sequence, count: usize) -> bool {
        self.buffer_size
            > (high_watermark - min_cursor_sequence(&self.gating_sequences)) as usize + count
    }
}

impl<W: WaitStrategy> Sequencer for MultiProducerSequencer<W> {
    type Barrier = ProcessingSequenceBarrier<W>;

    /// Claims the next sequence(s) in the ring buffer.
    ///
    /// This method will spin until it can claim the requested number of sequences.
    /// It uses atomic operations to ensure thread safety when multiple producers
    /// are requesting sequences concurrently.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of sequences to claim
    ///
    /// # Returns
    ///
    /// A tuple of (start_sequence, end_sequence) representing the claimed range
    #[inline(always)]
    fn next(&self, count: usize) -> (Sequence, Sequence) {
        loop {
            let high_watermark = self.high_watermark.get();
            if self.has_capacity(high_watermark, count) {
                let end = high_watermark + count as Sequence;
                if self.high_watermark.compare_exchange(high_watermark, end) {
                    return (high_watermark + 1, end);
                }
            }
        }
    }

    /// Publishes sequences to make them available to consumers.
    ///
    /// # Arguments
    ///
    /// * `lo` - The lowest sequence to publish
    /// * `hi` - The highest sequence to publish
    #[inline(always)]
    fn publish(&self, lo: Sequence, hi: Sequence) {
        for n in lo..=hi {
            self.ready_sequences.set(n);
        }

        let low_watermark = self.low_watermark.get();
        let mut good_to_release = low_watermark;

        while good_to_release < hi {
            if !self.ready_sequences.is_set(good_to_release + 1) {
                break;
            }
            good_to_release += 1;
        }

        if good_to_release > low_watermark {
            for n in low_watermark..=good_to_release {
                self.ready_sequences.unset(n);
            }

            let mut current = low_watermark;
            while !self.cursor.compare_exchange(current, good_to_release) {
                current = self.cursor.get();
                if current > good_to_release {
                    break;
                }
            }

            self.low_watermark.set(good_to_release);
            self.wait_strategy.signal();
        }
    }

    /// Creates a barrier for coordinating with consumers.
    ///
    /// # Arguments
    ///
    /// * `gating_sequences` - Sequences that the barrier should wait for
    ///
    /// # Returns
    ///
    /// A new processing sequence barrier
    fn create_barrier(
        &mut self,
        gating_sequences: &[Arc<AtomicSequence>],
    ) -> ProcessingSequenceBarrier<W> {
        ProcessingSequenceBarrier::new(
            self.wait_strategy.clone(),
            Vec::from(gating_sequences),
            self.is_done.clone(),
        )
    }

    /// Adds a gating sequence that this producer must wait for.
    ///
    /// # Arguments
    ///
    /// * `gating_sequence` - The sequence to add
    fn add_gating_sequence(&mut self, gating_sequence: &Arc<AtomicSequence>) {
        self.gating_sequences.push(gating_sequence.clone());
    }

    /// Gets the current cursor position.
    ///
    /// # Returns
    ///
    /// The current cursor as an atomic sequence
    fn get_cursor(&self) -> Arc<AtomicSequence> {
        self.cursor.clone()
    }

    /// Drains the sequencer, preventing further event production.
    fn drain(self) {
        let current = self.cursor.get();
        while min_cursor_sequence(&self.gating_sequences) < current {
            self.wait_strategy.signal();
        }
        self.is_done.store(true, Ordering::SeqCst);
        self.wait_strategy.signal();
    }
}
