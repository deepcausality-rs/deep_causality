// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::sequence::sequence::{AtomicSequence, Sequence};
use std::sync::Arc;

/// A trait that represents a barrier for a sequence.
///
/// A barrier is a blocking mechanism allowing to wait until a certain sequence
/// is available. It is used to synchronize the publisher and the consumer
/// threads.
///
/// A barrier is parameterized by a sequence and a gating sequence. The
/// `wait_for` method allows to wait until the sequence is greater than or equal
/// to the gating sequence. The `signal` method can be used to signal the barrier
/// that the sequence has changed.
pub trait SequenceBarrier: Send + Sync {
    /// Wait until the sequence is greater than or equal to the gating sequence.
    ///
    /// If the sequence is already greater than or equal to the gating sequence
    /// the method returns `Some(sequence)`, else it returns `None`.
    fn wait_for(&self, sequence: Sequence) -> Option<Sequence>;

    /// Signal the barrier that the sequence has changed.
    fn signal(&self);
}

/// A trait that represents a sequencer for a ring buffer.
///
/// A sequencer is an object used to manage the cursor of a ring buffer. It
/// provides methods to increment the cursor, to publish the cursor, to create
/// barriers for the cursor, and to add a gating sequence to the barriers.
pub trait Sequencer {
    type Barrier: SequenceBarrier;

    /// Increment the cursor by the given count.
    ///
    /// The method returns a tuple containing the new value of the cursor and
    /// the next value of the cursor.
    fn next(&self, count: usize) -> (Sequence, Sequence);

    /// Publish the cursor up to the given sequence.
    ///
    /// The method takes two sequences as arguments: the lower bound and the
    /// upper bound of the sequence range to publish.
    fn publish(&self, lo: Sequence, hi: Sequence);

    /// Create a barrier for the cursor.
    ///
    /// The method takes a slice of gating sequences as argument and returns a
    /// barrier that will block until the cursor is greater than or equal to
    /// any of the gating sequences.
    fn create_barrier(&mut self, gating_sequences: &[Arc<AtomicSequence>]) -> Self::Barrier;

    /// Add a gating sequence to the barriers.
    ///
    /// The method takes a gating sequence as argument and adds it to all the
    /// barriers.
    fn add_gating_sequence(&mut self, gating_sequence: &Arc<AtomicSequence>);

    /// Get the current value of the cursor.
    ///
    /// The method returns the current value of the cursor as an atomic sequence.
    fn get_cursor(&self) -> Arc<AtomicSequence>;

    /// Drain the sequencer.
    ///
    /// The method is used to release all the resources held by the sequencer.
    fn drain(self);
}
