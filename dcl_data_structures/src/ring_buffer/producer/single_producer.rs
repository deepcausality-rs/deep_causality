use crate::ring_buffer::prelude::*;
use std::cell::Cell;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct Producer<D: DataProvider<T>, T, S: Sequencer> {
    sequencer: S,
    data_provider: Arc<D>,
    _element: std::marker::PhantomData<T>,
}

pub struct SingleProducerSequencer<W: WaitStrategy> {
    cursor: Arc<AtomicSequence>,
    next_write_sequence: Cell<Sequence>,
    cached_available_sequence: Cell<Sequence>,
    wait_strategy: Arc<W>,
    gating_sequences: Vec<Arc<AtomicSequence>>,
    buffer_size: usize,
    is_done: Arc<AtomicBool>,
}

impl<W: WaitStrategy> SingleProducerSequencer<W> {
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

    fn publish(&self, _: Sequence, hi: Sequence) {
        self.cursor.set(hi);
        self.wait_strategy.signal();
    }

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

    fn add_gating_sequence(&mut self, gating_sequence: &Arc<AtomicSequence>) {
        self.gating_sequences.push(gating_sequence.clone());
    }

    fn get_cursor(&self) -> Arc<AtomicSequence> {
        self.cursor.clone()
    }

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
    fn drop(&mut self) {
        self.is_done.store(true, Ordering::SeqCst);
        self.wait_strategy.signal();
    }
}

impl<'a, D: DataProvider<T> + 'a, T, S: Sequencer + 'a> EventProducer<'a> for Producer<D, T, S> {
    type Item = T;

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

    fn drain(self) {
        self.sequencer.drain()
    }
}

impl<D: DataProvider<T>, T, S: Sequencer> Producer<D, T, S> {
    pub fn new(data_provider: Arc<D>, sequencer: S) -> Self {
        Producer {
            data_provider,
            sequencer,
            _element: Default::default(),
        }
    }
}
