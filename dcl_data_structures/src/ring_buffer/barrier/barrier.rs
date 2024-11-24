// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

//! # Sequence Barrier Implementation
//! 
//! This module provides a sequence barrier implementation for ring buffers, which is used to coordinate
//! multiple threads accessing a ring buffer in a lock-free manner. The barrier ensures proper
//! sequencing of operations and prevents data races between producers and consumers.
//!
//! ## Overview
//!
//! The sequence barrier acts as a coordination point between producers and consumers in a ring buffer,
//! ensuring that:
//! - Producers don't overwrite data that hasn't been processed by consumers
//! - Consumers don't read data that hasn't been fully written by producers
//!
//! ## Usage
//!
//! The barrier is typically used in conjunction with a ring buffer implementation where multiple
//! threads need to coordinate their access to shared data structures.
//!
/// # Type Parameters
///
/// * `W` - The wait strategy type that implements the `WaitStrategy` trait, determining how
///         threads should wait when they cannot proceed immediately.
///
/// # Fields
///
/// * `gating_sequences` - A vector of atomic sequences that this barrier depends on
/// * `wait_strategy` - The strategy used for waiting when sequences are not yet available
/// * `is_alerted` - A flag indicating whether the barrier has been alerted (typically for shutdown)
///
pub struct ProcessingSequenceBarrier<W: WaitStrategy> {
    gating_sequences: Vec<Arc<AtomicSequence>>,
    wait_strategy: Arc<W>,
    is_alerted: Arc<AtomicBool>,
}

impl<W: WaitStrategy> ProcessingSequenceBarrier<W> {
    /// Creates a new `ProcessingSequenceBarrier` with the specified parameters.
    ///
    /// # Parameters
    ///
    /// * `wait_strategy` - The strategy to use when waiting for sequences
    /// * `gating_sequences` - The sequences that this barrier depends on
    /// * `is_alerted` - A flag to indicate if the barrier has been alerted
    ///
    /// # Returns
    ///
    /// Returns a new instance of `ProcessingSequenceBarrier`
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
    /// Waits for a particular sequence to be available.
    ///
    /// This method will block the current thread until the requested sequence
    /// becomes available or until the barrier is alerted.
    ///
    /// # Parameters
    ///
    /// * `sequence` - The sequence number to wait for
    ///
    /// # Returns
    ///
    /// Returns `Some(Sequence)` if the sequence is available, or `None` if the
    /// barrier was alerted before the sequence became available.
    fn wait_for(&self, sequence: Sequence) -> Option<Sequence> {
        self.wait_strategy
            .wait_for(sequence, &self.gating_sequences, || {
                self.is_alerted.load(Ordering::Relaxed)
            })
    }

    /// Signals any waiting threads that they should check their conditions.
    ///
    /// This method is typically called when a condition that threads might be
    /// waiting for has changed.
    fn signal(&self) {
        self.wait_strategy.signal();
    }
}
