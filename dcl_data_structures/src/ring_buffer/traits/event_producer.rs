// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::sequence::atomic_sequence_ordered::Sequence;

/// A trait that represents a producer of events in a ring buffer.
///
/// The methods defined in this trait are used to write events to the buffer
/// and to drain the buffer once it is no longer needed.
///
/// The `write` method is used to write events to the buffer. It takes an iterator
/// and a closure as arguments. The closure is called for each item in the iterator
/// with a mutable reference to the item in the buffer, the sequence number of the
/// event and a reference to the item from the iterator.
///
/// The `drain` method is used to drain the buffer once it is no longer needed.
/// It takes no arguments and returns no result.
pub trait EventProducer<'a> {
    type Item;

    /// Writes events to the buffer.
    ///
    /// This method takes an iterator and a closure as arguments. The closure is
    /// called for each item in the iterator with a mutable reference to the item
    /// in the buffer, the sequence number of the event and a reference to the item
    /// from the iterator.
    ///
    /// The type parameter `I` is the type of the iterator, `U` is the type of
    /// the items in the iterator, `E` is the type of the iterator returned by
    /// `into_iter` and `F` is the type of the closure.
    ///
    /// The closure is called with the item in the buffer, the sequence number of
    /// the event and a reference to the item from the iterator as arguments.
    /// The closure is allowed to modify the item in the buffer.
    ///
    /// The method returns no value.
    fn write<F, U, I, E>(&self, items: I, f: F)
    where
        I: IntoIterator<Item = U, IntoIter = E>,
        E: ExactSizeIterator<Item = U>,
        F: Fn(&mut Self::Item, Sequence, &U);

    /// Drains the buffer once it is no longer needed.
    ///
    /// This method is intended to be called when the buffer is no longer required,
    /// and it will consume the `EventProducer` instance.
    fn drain(self);
}
