// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::sequence::Sequence;

/// A trait for atomic sequence operations
pub trait AtomicSequenceOps: Send + Sync + Clone {
    /// Create a new atomic sequence with the given initial value
    fn new(initial_value: Sequence) -> Self where Self: Sized;

    /// Create a new atomic sequence with initial value of 0
    fn default() -> Self where Self: Sized;

    /// Get the current sequence value
    fn get(&self) -> Sequence;

    /// Set the sequence value
    fn set(&self, value: Sequence);

    /// Compare and swap the sequence value
    fn compare_and_swap(&self, expected: Sequence, new: Sequence) -> Sequence;

    /// Increment the sequence value by 1
    fn increment(&self) -> Sequence;

    /// Add a value to the sequence
    fn add(&self, increment: Sequence) -> Sequence;

    /// Batch update the sequence
    fn batch_update(&self) -> Sequence;
}
