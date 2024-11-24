//! Single-producer implementation for sequential event production.
//!
//! This module provides a sequencer optimized for single-threaded event production.
//! It offers better performance than the multi-producer variant when only one thread
//! needs to produce events.
//!
//! # Example
//!
//! ```
//! use dcl_data_structures::ring_buffer::prelude::*;
//! use crossbeam_utils::atomic::AtomicCell;
//! use std::sync::Arc;
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
//! // Create a single producer
//! let buffer_size = 8;
//! let data_provider = Arc::new(MyDataProvider(
//!     (0..buffer_size).map(|_| AtomicCell::new(MyData(0))).collect()
//! ));
//! let wait_strategy = BlockingWaitStrategy::new();
//! let mut sequencer = SingleProducerSequencer::new(buffer_size, wait_strategy);
//! let producer = Producer::new(data_provider, sequencer);
//!
//! // Write some events
//! producer.write(vec![MyData(1), MyData(2)], |slot, _, item| {
//!     unsafe {
//!         (*slot).0 = item.0;
//!     }
//! });
//! ```

use crate::ring_buffer::prelude::*;
use std::cell::Cell;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// A producer that writes events to the ring buffer.
///
/// # Type Parameters
///
/// * `D` - The data provider type that manages the underlying storage
/// * `T` - The type of events being produced
/// * `S` - The sequencer type used for coordinating access
pub struct Producer<D: DataProvider<T>, T, S: Sequencer> {
    /// The sequencer used for coordinating access to the ring buffer
    sequencer: S,
    /// The data provider that manages the underlying storage
    data_provider: Arc<D>,
    /// Phantom data to track the event type
    _element: std::marker::PhantomData<T>,
}

/// A sequencer optimized for single-threaded event production.
///
/// This sequencer uses non-atomic operations where possible to achieve better
/// performance than the multi-producer implementation. It is not safe to use
/// from multiple threads simultaneously.
///
/// # Type Parameters
///
/// * `W` - The wait strategy used for coordinating with consumers
pub struct SingleProducerSequencer<W: WaitStrategy> {
    /// The current cursor position in the ring buffer
    cursor: Arc<AtomicSequence>,
    /// The next sequence to write to
    next_write_sequence: Cell<Sequence>,
    /// Cached sequence value to reduce consumer queries
    cached_available_sequence: Cell<Sequence>,
    /// The strategy used for waiting when the buffer is full
    wait_strategy: Arc<W>,
    /// Sequences that this producer must wait for before overwriting slots
    gating_sequences: Vec<Arc<AtomicSequence>>,
    /// Size of the ring buffer
    buffer_size: usize,
    /// Flag indicating if the sequencer has been drained
    is_done: Arc<AtomicBool>,
}

impl<W: WaitStrategy> SingleProducerSequencer<W> {
    /// Creates a new single-producer sequencer.
    ///
    /// # Arguments
    ///
    /// * `buffer_size` - The size of the ring buffer
    /// * `wait_strategy` - The strategy to use when waiting for available slots
    pub fn new(buffer_size: usize, wait_strategy: W) -> Self {
        SingleProducerSequencer {
            cursor: Arc::new(AtomicSequence::default()),
            next_write_sequence: Cell::new(0),
            cached_available_sequence: Cell::new(Sequence::default()),
            wait_strategy: Arc::new(wait_strategy),
            gating_sequences: Vec::new(),
            buffer_size,
            is_done: Default::default(),
        }
    }
}

impl<W: WaitStrategy> Sequencer for SingleProducerSequencer<W> {
    type Barrier = ProcessingSequenceBarrier<W>;

    /// Claims the next sequence(s) in the ring buffer.
    ///
    /// This method will block if there is insufficient space in the ring buffer.
    /// It uses non-atomic operations for better performance since it's designed
    /// for single-threaded use.
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
        let mut min_sequence = self.cached_available_sequence.take();
        let next = self.next_write_sequence.take();
        let (start, end) = (next, next + (count - 1) as Sequence);

        while min_sequence + (self.buffer_size as Sequence) < end {
            min_sequence = min_cursor_sequence(&self.gating_sequences);
        }

        self.cached_available_sequence.set(min_sequence);
        self.next_write_sequence.set(end + 1);

        (start, end)
    }

    /// Publishes sequences to make them available to consumers.
    ///
    /// # Arguments
    ///
    /// * `_` - The lowest sequence to publish (unused in single-producer case)
    /// * `hi` - The highest sequence to publish
    #[inline(always)]
    fn publish(&self, _: Sequence, hi: Sequence) {
        self.cursor.set(hi);
        self.wait_strategy.signal();
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
        let current = self.next_write_sequence.take() - 1;
        while min_cursor_sequence(&self.gating_sequences) < current {
            self.wait_strategy.signal();
        }
        self.is_done.store(true, Ordering::SeqCst);
        self.wait_strategy.signal();
    }
}

impl<W: WaitStrategy> Drop for SingleProducerSequencer<W> {
    /// Ensures proper cleanup when the sequencer is dropped.
    fn drop(&mut self) {
        self.is_done.store(true, Ordering::SeqCst);
        self.wait_strategy.signal();
    }
}

impl<'a, D: DataProvider<T> + 'a, T, S: Sequencer + 'a> EventProducer<'a> for Producer<D, T, S> {
    type Item = T;

    /// Writes items to the ring buffer.
    ///
    /// # Arguments
    ///
    /// * `items` - Iterator over the items to write
    /// * `f` - Function to apply to each item before writing
    fn write<F, U, I, E>(&self, items: I, f: F)
    where
        D: DataProvider<T>,
        I: IntoIterator<Item = U, IntoIter = E>,
        E: ExactSizeIterator<Item = U>,
        F: Fn(&mut Self::Item, Sequence, &U),
    {
        let iter = items.into_iter();
        let (start, end) = self.sequencer.next(iter.len());
        for (idx, item) in iter.enumerate() {
            let seq = start + idx as Sequence;
            let slot = unsafe { self.data_provider.get_mut(seq) };
            f(slot, seq, &item);
        }
        self.sequencer.publish(start, end);
    }

    /// Drains the producer, preventing further event production.
    fn drain(self) {
        self.sequencer.drain()
    }
}

impl<D: DataProvider<T>, T, S: Sequencer> Producer<D, T, S> {
    /// Creates a new producer.
    ///
    /// # Arguments
    ///
    /// * `data_provider` - The data provider that manages the underlying storage
    /// * `sequencer` - The sequencer used for coordinating access
    pub fn new(data_provider: Arc<D>, sequencer: S) -> Self {
        Producer {
            data_provider,
            sequencer,
            _element: Default::default(),
        }
    }
}
