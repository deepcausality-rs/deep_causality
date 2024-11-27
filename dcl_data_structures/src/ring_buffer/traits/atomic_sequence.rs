// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::prelude::Sequence;

/// A trait for atomic sequence operations
pub trait AtomicSequence: Send + Sync {

    /// Get the current sequence value
    fn get(&self) -> Sequence;

    /// Set the sequence value
    fn set(&self, value: Sequence);

    /// Compare and swap the sequence value
    fn compare_and_swap(&self, expected: Sequence, new: Sequence) -> bool;

    /// Increment the sequence value by 1
    fn increment(&self) -> Sequence;
}
