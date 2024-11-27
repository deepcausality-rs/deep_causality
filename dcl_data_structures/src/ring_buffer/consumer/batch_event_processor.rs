// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::marker::PhantomData;
use std::sync::Arc;

use crate::ring_buffer::prelude::*;

/// A batch event processor that processes events in sequence from a data provider.
///
/// The `BatchEventProcessor` provides functionality to process events in batches,
/// supporting both immutable and mutable event handling. It works in conjunction
/// with a sequence barrier to coordinate access to the event data and ensures
/// thread-safe processing of events.
///
/// # Features
/// - Supports both immutable and mutable event handlers
/// - Processes events in sequence using a cursor
/// - Thread-safe event processing
/// - Batch processing capability
///
/// # Examples
///
/// ```rust
/// use dcl_data_structures::ring_buffer::prelude::*;
/// use std::sync::Arc;
///
/// // Define a simple event type
/// #[derive(Debug)]
/// struct MyEvent(i32);
///
/// // Define an event handler
/// struct MyHandler;
/// impl EventHandler<MyEvent> for MyHandler {
///     fn handle_event(&self, event: &MyEvent, sequence: u64, end_of_batch: bool) {
///         println!("Processing event: {:?} at sequence {}", event, sequence);
///     }
/// }
///
/// // Create and use the processor
/// let handler = MyHandler;
/// let processor = BatchEventProcessor::create(handler);
/// ```
pub struct BatchEventProcessor;

impl BatchEventProcessor {
    /// Creates a new event processor with an immutable event handler.
    ///
    /// # Type Parameters
    /// * `'a`: Lifetime of the event processor
    /// * `E`: Type of the event handler
    /// * `T`: Type of the data being processed
    ///
    /// # Parameters
    /// * `handler`: The event handler implementation that will process events
    ///
    /// # Returns
    /// Returns an implementation of `EventProcessor` that can be used to process events
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dcl_data_structures::ring_buffer::prelude::*;
    /// use std::sync::Arc;
    ///
    /// struct MyEvent(i32);
    /// struct MyHandler;
    /// impl EventHandler<MyEvent> for MyHandler {
    ///     fn handle_event(&self, event: &MyEvent, sequence: u64, end_of_batch: bool) {
    ///         // Process the event
    ///     }
    /// }
    ///
    /// let handler = MyHandler;
    /// let processor = BatchEventProcessor::create(handler);
    /// ```
    pub fn create<'a, E, T>(handler: E) -> impl EventProcessor<'a, T>
    where
        T: Send + 'a,
        E: EventHandler<T> + Send + 'a,
    {
        Processor {
            handler,
            cursor: Default::default(),
            _marker: Default::default(),
        }
    }

    /// Creates a new event processor with a mutable event handler.
    ///
    /// # Type Parameters
    /// * `'a`: Lifetime of the event processor
    /// * `E`: Type of the event handler
    /// * `T`: Type of the data being processed
    ///
    /// # Parameters
    /// * `handler`: The mutable event handler implementation that will process events
    ///
    /// # Returns
    /// Returns an implementation of `EventProcessorMut` that can be used to process events
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dcl_data_structures::ring_buffer::prelude::*;
    /// use std::sync::Arc;
    ///
    /// struct MyEvent(i32);
    /// struct MyMutableHandler;
    /// impl EventHandlerMut<MyEvent> for MyMutableHandler {
    ///     fn handle_event(&mut self, event: &mut MyEvent, sequence: u64, end_of_batch: bool) {
    ///         // Process the event with mutable state
    ///     }
    /// }
    ///
    /// let handler = MyMutableHandler;
    /// let processor = BatchEventProcessor::create_mut(handler);
    /// ```
    pub fn create_mut<'a, E, T>(handler: E) -> impl EventProcessorMut<'a, T>
    where
        T: Send + 'a,
        E: EventHandlerMut<T> + Send + 'a,
    {
        ProcessorMut {
            handler,
            cursor: Default::default(),
            _marker: Default::default(),
        }
    }
}

/// An immutable event processor that processes events in sequence.
///
/// This processor maintains a cursor to track its progress through the event sequence
/// and uses an immutable event handler to process each event.
///
/// # Type Parameters
/// * `E`: Type of the event handler
/// * `T`: Type of the data being processed
struct Processor<E, T> {
    /// The event handler implementation
    handler: E,
    /// Cursor tracking the current position in the event sequence
    cursor: Arc<AtomicSequenceOrdered>,
    /// Phantom data to handle type parameters
    _marker: PhantomData<T>,
}

/// A mutable event processor that processes events in sequence.
///
/// This processor maintains a cursor to track its progress through the event sequence
/// and uses a mutable event handler to process each event.
///
/// # Type Parameters
/// * `E`: Type of the event handler
/// * `T`: Type of the data being processed
struct ProcessorMut<E, T> {
    /// The mutable event handler implementation
    handler: E,
    /// Cursor tracking the current position in the event sequence
    cursor: Arc<AtomicSequenceOrdered>,
    /// Phantom data to handle type parameters
    _marker: PhantomData<T>,
}

/// A runnable wrapper for an immutable processor.
///
/// This struct combines an immutable processor with its data provider and sequence barrier,
/// creating a runnable unit that can be executed to process events.
///
/// # Type Parameters
/// * `E`: Type of the event handler
/// * `T`: Type of the data being processed
/// * `D`: Type of the data provider
/// * `B`: Type of the sequence barrier
struct RunnableProcessor<E, T, D: DataProvider<T>, B: SequenceBarrier> {
    /// The underlying immutable processor
    processor: Processor<E, T>,
    /// The data provider that supplies events
    data_provider: Arc<D>,
    /// The sequence barrier that coordinates access to events
    barrier: B,
}

/// A runnable wrapper for a mutable processor.
///
/// This struct combines a mutable processor with its data provider and sequence barrier,
/// creating a runnable unit that can be executed to process events.
///
/// # Type Parameters
/// * `E`: Type of the event handler
/// * `T`: Type of the data being processed
/// * `D`: Type of the data provider
/// * `B`: Type of the sequence barrier
struct RunnableProcessorMut<E, T, D: DataProvider<T>, B: SequenceBarrier> {
    /// The underlying mutable processor
    processor: ProcessorMut<E, T>,
    /// The data provider that supplies events
    data_provider: Arc<D>,
    /// The sequence barrier that coordinates access to events
    barrier: B,
}

impl<'a, E, T> EventProcessorMut<'a, T> for Processor<E, T>
where
    E: EventHandler<T> + Send + 'a,
    T: Send + 'a,
{
    fn prepare<B: SequenceBarrier + 'a, D: DataProvider<T> + 'a>(
        self,
        barrier: B,
        data_provider: Arc<D>,
    ) -> Box<dyn Runnable + 'a> {
        Box::new(RunnableProcessor {
            processor: self,
            data_provider,
            barrier,
        })
    }

    fn get_cursor(&self) -> Arc<AtomicSequenceOrdered> {
        self.cursor.clone()
    }
}

impl<'a, E, T> EventProcessorMut<'a, T> for ProcessorMut<E, T>
where
    E: EventHandlerMut<T> + Send + 'a,
    T: Send + 'a,
{
    fn prepare<B: SequenceBarrier + 'a, D: DataProvider<T> + 'a>(
        self,
        barrier: B,
        data_provider: Arc<D>,
    ) -> Box<dyn Runnable + 'a> {
        Box::new(RunnableProcessorMut {
            processor: self,
            data_provider,
            barrier,
        })
    }

    fn get_cursor(&self) -> Arc<AtomicSequenceOrdered> {
        self.cursor.clone()
    }
}

impl<'a, E, T> EventProcessor<'a, T> for Processor<E, T>
where
    E: EventHandler<T> + Send + 'a,
    T: Send + 'a,
{
}

impl<E, T, D, B> Runnable for RunnableProcessor<E, T, D, B>
where
    E: EventHandler<T> + Send,
    D: DataProvider<T>,
    B: SequenceBarrier,
    T: Send,
{
    fn run(mut self: Box<Self>) {
        let f = &mut self.processor.handler;
        let cursor = &self.processor.cursor;
        let data_provider = &self.data_provider;
        let barrier = &self.barrier;

        loop {
            let next = cursor.get() + 1;
            let available = match barrier.wait_for(next) {
                Some(seq) => seq,
                None => return,
            };

            for i in next..=available {
                let value = unsafe { data_provider.get(i) };
                f.handle_event(value, i, i == available);
            }

            cursor.set(available);
            barrier.signal();
        }
    }
}

impl<E, T, D, B> Runnable for RunnableProcessorMut<E, T, D, B>
where
    E: EventHandlerMut<T> + Send,
    D: DataProvider<T>,
    B: SequenceBarrier,
    T: Send,
{
    fn run(mut self: Box<Self>) {
        let f = &mut self.processor.handler;
        let cursor = &self.processor.cursor;
        let data_provider = &self.data_provider;
        let barrier = &self.barrier;

        loop {
            let next = cursor.get() + 1;
            let available = match barrier.wait_for(next) {
                Some(seq) => seq,
                None => return,
            };

            for i in next..=available {
                let value = unsafe { data_provider.get_mut(i) };
                f.handle_event(value, i, i == available);
            }

            cursor.set(available);
            barrier.signal();
        }
    }
}
