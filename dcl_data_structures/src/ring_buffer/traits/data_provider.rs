// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::sequence::atomic_sequence::Sequence;

#[allow(clippy::mut_from_ref)]
/// A trait representing a data provider for a ring buffer.
///
/// This trait defines methods to access and modify buffer elements using sequences.
pub trait DataProvider<T>: Sync + Send {
    /// Returns the size of the buffer.
    fn buffer_size(&self) -> usize;

    /// Provides mutable access to the element at the given sequence.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows mutable access to
    /// potentially shared data, which can lead to data races if used improperly.
    unsafe fn get_mut(&self, sequence: Sequence) -> &mut T;

    /// Provides immutable access to the element at the given sequence.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it allows access to
    /// potentially shared data, which can lead to data races if used improperly.
    unsafe fn get(&self, sequence: Sequence) -> &T;
}
