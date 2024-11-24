// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::marker::PhantomData;
use std::sync::Arc;

use crate::ring_buffer::prelude::*;

/// A batch event processor that processes events in batches.
///
/// This processor supports both immutable and mutable event handling through
/// the `create` and `create_mut` methods respectively.
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
    /// * `handler`: The event handler implementation
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
    /// * `handler`: The mutable event handler implementation
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
struct Processor<E, T> {
    handler: E,
    cursor: Arc<AtomicSequence>,
    _marker: PhantomData<T>,
}

/// A mutable event processor that processes events in sequence.
struct ProcessorMut<E, T> {
    handler: E,
    cursor: Arc<AtomicSequence>,
    _marker: PhantomData<T>,
}

/// A runnable wrapper for an immutable processor.
struct RunnableProcessor<E, T, D: DataProvider<T>, B: SequenceBarrier> {
    processor: Processor<E, T>,
    data_provider: Arc<D>,
    barrier: B,
}

/// A runnable wrapper for a mutable processor.
struct RunnableProcessorMut<E, T, D: DataProvider<T>, B: SequenceBarrier> {
    processor: ProcessorMut<E, T>,
    data_provider: Arc<D>,
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

    fn get_cursor(&self) -> Arc<AtomicSequence> {
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

    fn get_cursor(&self) -> Arc<AtomicSequence> {
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
