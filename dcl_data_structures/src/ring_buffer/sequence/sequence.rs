// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

//! # Atomic Sequence Implementation for Ring Buffers
//!
//! This module provides an atomic sequence implementation optimized for ring buffer operations.
//! It includes cache-line padding to prevent false sharing in concurrent scenarios and atomic
//! operations for thread-safe sequence management.
//!
//! ## Overview
//!
//! The sequence implementation consists of two main components:
//! - A type alias `Sequence` representing sequence numbers as `u64`
//! - An `AtomicSequence` struct providing atomic operations on sequences with cache-line padding
//!
//! ## Cache-Line Optimization
//!
//! The implementation uses cache-line padding to prevent false sharing between different
//! `AtomicSequence` instances when used in concurrent scenarios. The padding size is
//! architecture-specific:
//! - 128 bytes for Apple Silicon (aarch64)
//! - 64 bytes for x86_64 architectures
//!
//! ## Usage Example
//!
//! ```rust
//! use dcl_data_structures::ring_buffer::prelude::*;
//!
//! // Create a new atomic sequence with default value (0)
//! let seq = AtomicSequence::default();
//!
//! // Set a sequence value
//! seq.set(42);
//!
//! // Get the current value
//! assert_eq!(seq.get(), 42);
//!
//! // Perform a compare-exchange operation
//! let success = seq.compare_exchange(42, 43);
//! assert!(success);
//! assert_eq!(seq.get(), 43);
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::mem::size_of;

/// Type alias for sequence numbers in the ring buffer.
/// Uses u64 to provide a large range of sequence numbers before wrapping.
pub type Sequence = u64;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const CACHE_LINE_SIZE: usize = 128;

#[cfg(target_arch = "x86_64")]
const CACHE_LINE_SIZE: usize = 64;

const CACHE_LINE_PADDING: usize = CACHE_LINE_SIZE - size_of::<AtomicU64>();

/// An atomic sequence with cache-line padding to prevent false sharing.
///
/// This struct is designed to be used in concurrent scenarios where multiple threads
/// may be accessing different sequence numbers simultaneously. The cache-line padding
/// ensures that modifications to one sequence don't invalidate cache lines containing
/// other sequences.
///
/// # Memory Layout
///
/// The struct is aligned to 64 bytes and contains:
/// - Padding bytes to fill a cache line
/// - An atomic u64 for the sequence value
///
/// # Thread Safety
///
/// All operations on `AtomicSequence` are atomic and thread-safe, using appropriate
/// memory ordering guarantees:
/// - `get` uses Acquire ordering
/// - `set` uses Release ordering
/// - `compare_exchange` uses SeqCst for success and Acquire for failure
#[repr(align(64))]
pub struct AtomicSequence {
    _pad: [u8; CACHE_LINE_PADDING],
    offset: AtomicU64,
}

impl AtomicSequence {
    /// Atomically loads and returns the current sequence value.
    ///
    /// Uses Acquire ordering to ensure visibility of values written by other threads.
    ///
    /// # Returns
    ///
    /// The current sequence value
    pub fn get(&self) -> Sequence {
        self.offset.load(Ordering::Acquire)
    }

    /// Atomically stores a new sequence value.
    ///
    /// Uses Release ordering to ensure other threads will see this write.
    ///
    /// # Parameters
    ///
    /// * `value` - The new sequence value to store
    pub fn set(&self, value: Sequence) {
        self.offset.store(value, Ordering::Release);
    }

    /// Atomically compares and exchanges sequence values.
    ///
    /// Compares the current value with `current` and, if equal, replaces it with `new`.
    /// Uses SeqCst ordering for success and Acquire for failure to ensure strong consistency.
    ///
    /// # Parameters
    ///
    /// * `current` - The value to compare against
    /// * `new` - The value to store if comparison succeeds
    ///
    /// # Returns
    ///
    /// `true` if the exchange was successful, `false` otherwise
    pub fn compare_exchange(&self, current: Sequence, new: Sequence) -> bool {
        self.offset
            .compare_exchange(current, new, Ordering::SeqCst, Ordering::Acquire)
            .is_ok()
    }
}

impl Default for AtomicSequence {
    /// Creates a new `AtomicSequence` with a default value of 0.
    ///
    /// # Returns
    ///
    /// A new `AtomicSequence` instance initialized to 0
    fn default() -> Self {
        Self {
            _pad: [0; CACHE_LINE_PADDING],
            offset: AtomicU64::default(),
        }
    }
}

impl From<Sequence> for AtomicSequence {
    /// Creates a new `AtomicSequence` from a sequence value.
    ///
    /// # Parameters
    ///
    /// * `value` - The initial sequence value
    ///
    /// # Returns
    ///
    /// A new `AtomicSequence` initialized with the given value
    fn from(value: Sequence) -> Self {
        Self {
            _pad: [0; CACHE_LINE_PADDING],
            offset: AtomicU64::new(value),
        }
    }
}

impl From<AtomicSequence> for Sequence {
    /// Converts an `AtomicSequence` into its raw sequence value.
    ///
    /// # Parameters
    ///
    /// * `val` - The `AtomicSequence` to convert
    ///
    /// # Returns
    ///
    /// The underlying sequence value
    fn from(val: AtomicSequence) -> Self {
        val.offset.into_inner()
    }
}
