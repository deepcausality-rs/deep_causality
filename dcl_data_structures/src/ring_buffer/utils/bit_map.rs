// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::sequence::atomic_sequence_ordered::Sequence;
use crate::ring_buffer::utils::logarithm::log2;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};

const WORD_BITS: usize = size_of::<AtomicU64>() * 8;

/// A thread-safe bitmap implementation using atomic operations for concurrent access.
///
/// `BitMap` provides a fixed-size bit array that can be safely accessed and modified
/// from multiple threads. It uses atomic operations to ensure thread safety without
/// requiring explicit locks.
///
/// # Implementation Details
///
/// - Uses an array of `AtomicU64` for storage, where each `AtomicU64` represents 64 bits
/// - Employs bit manipulation for efficient storage and retrieval
/// - All operations are atomic and thread-safe
/// - Memory efficient: bits are packed into 64-bit words
///
/// # Capacity
///
/// The bitmap's capacity is rounded up to the nearest multiple of 64 bits (word size).
/// The actual storage size is calculated as `(capacity + 63) / 64` words.
///
/// # Thread Safety
///
/// All operations use `SeqCst` ordering to ensure strict consistency in concurrent scenarios.
pub struct BitMap {
    slots: Box<[AtomicU64]>,
    index_mask: u64,
    index_shift: u64,
    word_bits_mask: usize,
}

impl BitMap {
    /// Creates a new `BitMap` with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of bits to store, must be non-zero
    ///
    /// # Returns
    ///
    /// A new `BitMap` instance initialized with all bits set to zero
    pub fn new(capacity: NonZeroUsize) -> BitMap {
        Self::build(capacity)
    }

    fn build(capacity: NonZeroUsize) -> BitMap {
        let len = (capacity.get() + WORD_BITS - 1) / WORD_BITS;

        let slots = std::iter::repeat_with(AtomicU64::default)
            .take(len)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let index_mask = (capacity.get() - 1) as u64;
        let index_shift = log2(WORD_BITS as u64);
        let word_bits_mask = WORD_BITS - 1;

        Self {
            slots,
            index_mask,
            index_shift,
            word_bits_mask,
        }
    }
}

impl BitMap {
    /// Checks if a bit is set at the specified sequence number.
    ///
    /// # Arguments
    ///
    /// * `sequence` - The sequence number to check
    ///
    /// # Returns
    ///
    /// `true` if the bit is set, `false` otherwise
    ///
    /// # Safety
    ///
    /// Uses unchecked array access for performance. The safety is guaranteed by
    /// the index masking operation that ensures the index is within bounds.
    pub fn is_set(&self, sequence: Sequence) -> bool {
        let index = (sequence & self.index_mask >> self.index_shift) as usize;
        let slot = unsafe { self.slots.get_unchecked(index) };
        let val = slot.load(Ordering::SeqCst);
        val & (1 << (index & self.word_bits_mask)) != 0
    }

    /// Sets the bit at the specified sequence number.
    ///
    /// # Arguments
    ///
    /// * `sequence` - The sequence number where the bit should be set
    ///
    /// # Safety
    ///
    /// Uses unchecked array access for performance. The safety is guaranteed by
    /// the index masking operation that ensures the index is within bounds.
    pub fn set(&self, sequence: Sequence) {
        let index = (sequence & self.index_mask >> self.index_shift) as usize;
        let slot = unsafe { self.slots.get_unchecked(index) };
        let val = 1 << (index & self.word_bits_mask);
        slot.fetch_or(val, Ordering::SeqCst);
    }

    /// Unsets (clears) the bit at the specified sequence number.
    ///
    /// # Arguments
    ///
    /// * `sequence` - The sequence number where the bit should be unset
    ///
    /// # Safety
    ///
    /// Uses unchecked array access for performance. The safety is guaranteed by
    /// the index masking operation that ensures the index is within bounds.
    pub fn unset(&self, sequence: Sequence) {
        let index = (sequence & self.index_mask >> self.index_shift) as usize;
        let slot = unsafe { self.slots.get_unchecked(index) };
        let val = !(1 << (index & self.word_bits_mask));
        slot.fetch_and(val, Ordering::SeqCst);
    }
}
