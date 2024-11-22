use crate::prelude::*;
use crate::ring_buffer::sequence::sequence::{AtomicSequence, Sequence};
use crate::ring_buffer::traits::sequencer::SequenceBarrier;
use crate::ring_buffer::traits::wait_strategy::WaitStrategy;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct ProcessingSequenceBarrier<W: WaitStrategy> {
    // Replace this vector
    gating_sequences: Vec<Arc<AtomicSequence>>,
    wait_strategy: Arc<W>,
    is_alerted: Arc<AtomicBool>,
}

impl<W: WaitStrategy> ProcessingSequenceBarrier<W> {
    pub fn new(
        wait_strategy: Arc<W>,
        gating_sequences: Vec<Arc<AtomicSequence>>,
        is_alerted: Arc<AtomicBool>,
    ) -> Self {
        ProcessingSequenceBarrier {
            wait_strategy,
            gating_sequences,
            is_alerted,
        }
    }
}

impl<W: WaitStrategy> SequenceBarrier for ProcessingSequenceBarrier<W> {
    fn wait_for(&self, sequence: Sequence) -> Option<Sequence> {
        self.wait_strategy
            .wait_for(sequence, &self.gating_sequences, || {
                self.is_alerted.load(Ordering::Relaxed)
            })
    }

    fn signal(&self) {
        self.wait_strategy.signal();
    }
}
