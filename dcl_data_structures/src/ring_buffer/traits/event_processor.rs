// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::ring_buffer::sequence::sequence::AtomicSequence;
use crate::ring_buffer::traits::data_provider::DataProvider;
use crate::ring_buffer::traits::runnable::Runnable;
use crate::ring_buffer::traits::sequencer::SequenceBarrier;
use std::sync::Arc;

/// A trait for event processors that can be used for event processing.
///
/// An event processor is responsible for processing events in a ring buffer.
/// It receives events from the ring buffer and processes them in an order
/// determined by the sequence of events in the buffer.
///
/// The trait provides methods to prepare the event processor with a sequence
/// barrier and a data provider, and to retrieve the cursor sequence of the
/// event processor.
pub trait EventProcessorMut<'a, T> {
    /// Prepares the event processor with a sequence barrier and a data provider.
    ///
    /// The sequence barrier is used to determine the next available event to
    /// process, and the data provider is used to retrieve the event from the
    /// ring buffer.
    fn prepare<B: SequenceBarrier + 'a, D: DataProvider<T> + 'a>(
        self,
        barrier: B,
        data_provider: Arc<D>,
    ) -> Box<dyn Runnable + 'a>;

    /// Retrieves the cursor sequence of the event processor.
    ///
    /// The cursor sequence is used to track the sequence of events that have
    /// been processed by the event processor.
    fn get_cursor(&self) -> Arc<AtomicSequence>;
}

/// A trait for event processors that can be used for event processing.
///
/// An event processor is responsible for processing events in a ring buffer.
/// It receives events from the ring buffer and processes them in an order
/// determined by the sequence of events in the buffer.
///
/// The trait provides methods to prepare the event processor with a sequence
/// barrier and a data provider, and to retrieve the cursor sequence of the
/// event processor.
pub trait EventProcessor<'a, T>: EventProcessorMut<'a, T> {}

/// A trait for executing event processors and managing their lifecycle.
///
/// This trait provides methods to create an executor with runnables and to
/// spawn the execution, returning a handle to manage the execution.
pub trait EventProcessorExecutor<'a> {
    /// Type of handle returned by the executor to manage execution.
    type Handle: ExecutorHandle;

    /// Create a new executor with the specified runnables.
    ///
    /// # Arguments
    ///
    /// * `items` - A vector of boxed runnables to be executed.
    fn with_runnables(items: Vec<Box<dyn Runnable + 'a>>) -> Self;

    /// Spawn the execution of the runnables.
    ///
    /// Returns a handle to manage the execution lifecycle.
    fn spawn(self) -> Self::Handle;
}

/// A handle to an event processor execution.
///
/// The handle is used to join the execution in order to wait until
/// all the tasks have finished.
pub trait ExecutorHandle {
    /// Join the execution of the event processor.
    ///
    /// This method blocks until all the tasks have finished.
    fn join(self);
}
