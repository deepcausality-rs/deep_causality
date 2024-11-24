use crate::ring_buffer::prelude::*;
use std::num::NonZeroUsize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct MultiProducerSequencer<W: WaitStrategy> {
    cursor: Arc<AtomicSequence>,
    wait_strategy: Arc<W>,
    gating_sequences: Vec<Arc<AtomicSequence>>,
    buffer_size: usize,
    high_watermark: AtomicSequence,
    ready_sequences: BitMap,
    is_done: Arc<AtomicBool>,
}

impl<W: WaitStrategy> MultiProducerSequencer<W> {
    pub fn new(buffer_size: usize, wait_strategy: W) -> Self {
        MultiProducerSequencer {
            cursor: Arc::new(AtomicSequence::default()),
            wait_strategy: Arc::new(wait_strategy),
            gating_sequences: Vec::new(),
            buffer_size,
            high_watermark: AtomicSequence::default(),
            ready_sequences: BitMap::new(NonZeroUsize::try_from(buffer_size).unwrap()),
            is_done: Default::default(),
        }
    }

    fn has_capacity(&self, high_watermark: Sequence, count: usize) -> bool {
        self.buffer_size
            > (high_watermark - min_cursor_sequence(&self.gating_sequences)) as usize + count
    }
}

impl<W: WaitStrategy> Sequencer for MultiProducerSequencer<W> {
    type Barrier = ProcessingSequenceBarrier<W>;

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

    fn publish(&self, lo: Sequence, hi: Sequence) {
        for n in lo..=hi {
            self.ready_sequences.set(n);
        }

        let low_watermark = self.cursor.get() + 1;
        let mut good_to_release = low_watermark - 1;
        for n in low_watermark..=self.high_watermark.get() {
            if self.ready_sequences.is_set(n) {
                good_to_release = n;
            } else {
                break;
            }
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
        }

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
        let current = self.cursor.get();
        while min_cursor_sequence(&self.gating_sequences) < current {
            self.wait_strategy.signal();
        }
        self.is_done.store(true, Ordering::SeqCst);
        self.wait_strategy.signal();
    }
}
