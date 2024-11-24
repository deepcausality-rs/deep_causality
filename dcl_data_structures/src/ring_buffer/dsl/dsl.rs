use std::{marker::PhantomData, sync::Arc};

use crate::ring_buffer::prelude::*;

/// Creates a new RingBuffer with the specified capacity.
/// The capacity must be a power of two.
#[derive(Debug)]
pub struct RustDisruptorBuilder {}

pub struct WithDataProvider<D: DataProvider<T>, T>
where
    T: Send + Sync,
{
    data_provider: Arc<D>,
    _element: PhantomData<T>,
}

pub struct WithWaitStrategy<W: WaitStrategy, D: DataProvider<T>, T>
where
    T: Send + Sync,
{
    with_data_provider: WithDataProvider<D, T>,
    _wait_strategy: PhantomData<W>,
}

pub struct WithSequencer<S: Sequencer, W: WaitStrategy, D: DataProvider<T>, T>
where
    T: Send + Sync,
{
    with_wait_strategy: WithWaitStrategy<W, D, T>,
    sequencer: S,
}

pub struct BarrierScope<'a, S: Sequencer, D: DataProvider<T>, T> {
    sequencer: S,
    data_provider: Arc<D>,
    gating_sequences: Vec<Arc<AtomicSequence>>,
    cursors: Vec<Arc<AtomicSequence>>,
    event_handlers: Vec<Box<dyn Runnable + 'a>>,
    _element: PhantomData<T>,
}

pub struct WithEventHandlers<'a, S: Sequencer, W: WaitStrategy, D: DataProvider<T>, T>
where
    T: Send + Sync,
{
    with_sequencer: WithSequencer<S, W, D, T>,
    event_handlers: Vec<Box<dyn Runnable + 'a>>,
    gating_sequences: Vec<Arc<AtomicSequence>>,
}

impl RustDisruptorBuilder {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<D: DataProvider<T>, T>(data_provider: Arc<D>) -> WithDataProvider<D, T>
    where
        T: Send + Sync,
    {
        WithDataProvider {
            data_provider,
            _element: Default::default(),
        }
    }

    pub fn with_ring_buffer<T, const N: usize>(capacity: usize) -> WithDataProvider<RingBuffer<T, N>, T>
    where
        T: Default + Copy + Send + Sync,
    {
        assert!(
            (capacity != 0) && ((capacity & (capacity - 1)) == 0),
            "capacity must be power of two"
        );
        Self::new(Arc::new(RingBuffer::new()))
    }
}

impl<D: DataProvider<T>, T> WithDataProvider<D, T>
where
    T: Send + Sync,
{
    pub fn with_wait_strategy<W: WaitStrategy>(self) -> WithWaitStrategy<W, D, T> {
        WithWaitStrategy {
            with_data_provider: self,
            _wait_strategy: Default::default(),
        }
    }

    pub fn with_blocking_wait(self) -> WithWaitStrategy<BlockingWaitStrategy, D, T> {
        self.with_wait_strategy()
    }

    pub fn with_spin_wait(self) -> WithWaitStrategy<SpinLoopWaitStrategy, D, T> {
        self.with_wait_strategy()
    }
}

impl<W: WaitStrategy, D: DataProvider<T>, T> WithWaitStrategy<W, D, T>
where
    T: Send + Sync,
{
    pub fn with_sequencer<S: Sequencer>(self, sequencer: S) -> WithSequencer<S, W, D, T> {
        WithSequencer {
            with_wait_strategy: self,
            sequencer,
        }
    }

    pub fn with_single_producer(self) -> WithSequencer<SingleProducerSequencer<W>, W, D, T> {
        let buffer_size = self.with_data_provider.data_provider.buffer_size();
        self.with_sequencer(SingleProducerSequencer::new(buffer_size, W::new()))
    }

    pub fn with_multi_producer(self) -> WithSequencer<MultiProducerSequencer<W>, W, D, T> {
        let buffer_size = self.with_data_provider.data_provider.buffer_size();
        self.with_sequencer(MultiProducerSequencer::new(buffer_size, W::new()))
    }
}

impl<'a, S: Sequencer + 'a, W: WaitStrategy, D: DataProvider<T> + 'a, T: Send + Sync + 'a>
WithSequencer<S, W, D, T>
{
    pub fn with_barrier(
        mut self,
        f: impl FnOnce(&mut BarrierScope<'a, S, D, T>),
    ) -> WithEventHandlers<'a, S, W, D, T> {
        let cursor = self.sequencer.get_cursor();
        let mut scope = BarrierScope {
            sequencer: self.sequencer,
            data_provider: self
                .with_wait_strategy
                .with_data_provider
                .data_provider
                .clone(),
            gating_sequences: vec![cursor],
            event_handlers: Vec::new(),
            cursors: Vec::new(),
            _element: Default::default(),
        };

        f(&mut scope);
        self.sequencer = scope.sequencer;

        WithEventHandlers {
            with_sequencer: self,
            event_handlers: scope.event_handlers,
            gating_sequences: scope.cursors,
        }
    }
}

impl<'a, S: Sequencer + 'a, D: DataProvider<T> + 'a, T: Send + 'a> BarrierScope<'a, S, D, T> {
    pub fn handle_events<E>(&mut self, handler: E)
    where
        E: EventHandler<T> + Send + 'a,
    {
        self.handle_events_with(BatchEventProcessor::create(handler))
    }

    pub fn handle_events_mut<E>(&mut self, handler: E)
    where
        E: EventHandlerMut<T> + Send + 'a,
    {
        self.handle_events_with(BatchEventProcessor::create_mut(handler))
    }

    pub fn handle_events_with<E: EventProcessorMut<'a, T>>(&mut self, processor: E) {
        self.cursors.push(processor.get_cursor());
        let barrier = self.sequencer.create_barrier(&self.gating_sequences);

        let runnable = processor.prepare(barrier, self.data_provider.clone());
        self.event_handlers.push(runnable);
    }

    pub fn with_barrier(mut self, f: impl FnOnce(&mut BarrierScope<'a, S, D, T>)) {
        let mut scope = BarrierScope {
            sequencer: self.sequencer,
            data_provider: self.data_provider.clone(),
            gating_sequences: self.cursors,
            event_handlers: Vec::new(),
            cursors: Vec::new(),
            _element: Default::default(),
        };

        f(&mut scope);
        self.event_handlers.append(&mut scope.event_handlers);
    }
}

impl<'a, S: Sequencer + 'a, W: WaitStrategy, D: DataProvider<T> + 'a, T: Send + Sync + 'a>
WithEventHandlers<'a, S, W, D, T>
{
    pub fn with_barrier(mut self, f: impl FnOnce(&mut BarrierScope<'a, S, D, T>)) -> Self {
        let mut scope = BarrierScope {
            gating_sequences: self.gating_sequences.clone(),
            cursors: Vec::new(),
            sequencer: self.with_sequencer.sequencer,
            data_provider: self
                .with_sequencer
                .with_wait_strategy
                .with_data_provider
                .data_provider
                .clone(),
            event_handlers: Vec::new(),
            _element: Default::default(),
        };

        f(&mut scope);
        self.with_sequencer.sequencer = scope.sequencer;
        self.event_handlers.append(&mut scope.event_handlers);
        self.gating_sequences = scope.cursors;

        self
    }

    pub fn build(
        self,
    ) -> (
        impl EventProcessorExecutor<'a>,
        impl EventProducer<'a, Item = T>,
    ) {
        self.build_with_executor::<ThreadedExecutor<'a>>()
    }

    pub fn build_with_executor<E: EventProcessorExecutor<'a>>(
        mut self,
    ) -> (E, impl EventProducer<'a, Item = T>) {
        for gs in &self.gating_sequences {
            self.with_sequencer.sequencer.add_gating_sequence(gs);
        }
        let executor = E::with_runnables(self.event_handlers);
        let producer = Producer::new(
            self.with_sequencer
                .with_wait_strategy
                .with_data_provider
                .data_provider
                .clone(),
            self.with_sequencer.sequencer,
        );
        (executor, producer)
    }
}