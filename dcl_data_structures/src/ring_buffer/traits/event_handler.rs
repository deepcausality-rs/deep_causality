// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::prelude::Sequence;

pub trait EventHandler<T> {
    /// Handle an event from the ring buffer.
    ///
    /// The event is referenced by &T and not &mut T because the event processor
    /// may choose to cache the event in case the event handler fails to process
    /// the event.  If the event handler fails to process the event, the event
    /// processor will not advance the sequence and the event handler will be
    /// called again with the same event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event from the ring buffer
    /// * `sequence` - The sequence number of the event
    /// * `eob` - Whether this event is the last event in the batch
    fn handle_event(&self, event: &T, sequence: Sequence, eob: bool);
}

pub trait EventHandlerMut<T> {
    /// Handle an event from the ring buffer.
    ///
    /// The event is referenced by &mut T, meaning the event handler can modify
    /// the event in case the event handler fails to process the event.  If the
    /// event handler fails to process the event, the event processor will not
    /// advance the sequence and the event handler will be called again with the
    /// same event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event from the ring buffer
    /// * `sequence` - The sequence number of the event
    /// * `eob` - Whether this event is the last event in the batch
    fn handle_event(&mut self, event: &mut T, sequence: Sequence, eob: bool);
}
