// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::prelude::{AtomicSequence, Sequence};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

const MIN_BATCH_SIZE: u64 = 32;
const CACHE_LINE_PADDING: usize = 56;

/// An optimized atomic sequence with relaxed memory ordering and adaptive batch sizing.
///
/// This implementation focuses on high performance for multi-producer scenarios by:
/// - Using relaxed memory ordering where possible
/// - Implementing adaptive batch sizing based on contention
/// - Employing cache-line padding to prevent false sharing
/// - Using an optimized backoff strategy
#[repr(align(64))]
#[derive(Debug)]
pub struct AtomicSequenceRelaxed {
    _pad: [u8; CACHE_LINE_PADDING],
    offset: AtomicU64,
    contention_counter: AtomicUsize,
    current_batch_size: AtomicU64,
}

impl Default for AtomicSequenceRelaxed {
    fn default() -> Self {
        Self {
            _pad: [0; CACHE_LINE_PADDING],
            offset: AtomicU64::default(),
            contention_counter: AtomicUsize::default(),
            current_batch_size: AtomicU64::new(MIN_BATCH_SIZE),
        }
    }
}

impl Clone for AtomicSequenceRelaxed {
    fn clone(&self) -> Self {
        Self {
            _pad: self._pad,
            offset: AtomicU64::new(self.offset.load(Ordering::Relaxed)),
            contention_counter: AtomicUsize::new(self.contention_counter.load(Ordering::Relaxed)),
            current_batch_size: AtomicU64::new(self.current_batch_size.load(Ordering::Relaxed)),
        }
    }
}

impl AtomicSequence for AtomicSequenceRelaxed {
    fn get(&self) -> Sequence {
        self.offset.load(Ordering::Relaxed) as Sequence
    }

    fn set(&self, value: Sequence) {
        self.offset.store(value, Ordering::Relaxed);
    }

    fn compare_and_swap(&self, current: Sequence, new: Sequence) -> bool {
        self.offset
            .compare_exchange(current, new, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    }

    fn increment(&self) -> Sequence {
        self.offset.fetch_add(1, Ordering::Relaxed) as Sequence
    }
}

impl From<Sequence> for AtomicSequenceRelaxed {
    fn from(value: Sequence) -> Self {
        Self {
            _pad: [0; CACHE_LINE_PADDING],
            offset: AtomicU64::new(value),
            contention_counter: AtomicUsize::new(0),
            current_batch_size: AtomicU64::new(MIN_BATCH_SIZE),
        }
    }
}

impl From<AtomicSequenceRelaxed> for Sequence {
    fn from(val: AtomicSequenceRelaxed) -> Self {
        val.offset.into_inner() as Sequence
    }
}
