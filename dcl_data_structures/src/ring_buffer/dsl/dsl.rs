use std::{marker::PhantomData, sync::Arc};

use crate::ring_buffer::prelude::*;

/// A builder pattern DSL (Domain Specific Language) for constructing and configuring
/// a high-performance ring buffer system inspired by the LMAX Disruptor pattern.
///
/// This DSL provides a fluent interface for:
/// - Creating ring buffers with specific configurations
/// - Setting up wait strategies
/// - Configuring producers (single or multi)
/// - Setting up event handlers and processors
/// - Managing barrier sequences
///
/// # Examples
/// ```
/// use dcl_data_structures::ring_buffer::prelude::*;
/// use std::sync::Arc;
///
/// // Define an immutable event handler
/// struct PrintHandler;
/// impl EventHandler<u64> for PrintHandler {
///     fn handle_event(&self, event: &u64, sequence: u64, end_of_batch: bool) {
///         println!("Received: {} at sequence {}", event, sequence);
///     }
/// }
///
/// // Define a mutable event handler
/// struct DoubleHandler;
/// impl EventHandlerMut<i32> for DoubleHandler {
///     fn handle_event(&mut self, event: &mut i32, sequence: u64, end_of_batch: bool) {
///         *event *= 2;  // Double the value
///         if end_of_batch {
///             println!("Processed sequence {}", sequence);
///         }
///     }
/// }
///
/// // Create a ring buffer with single producer
/// let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
///     .with_blocking_wait()
///     .with_single_producer()
///     .with_barrier(|scope| {
///         // Handle events immutably
///         scope.handle_events(PrintHandler);
///     })
///     .build();
///
/// // Create a ring buffer with multiple producers and custom wait strategy
/// let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<i32, 2048>(2048)
///     .with_spin_wait()
///     .with_multi_producer()
///     .with_barrier(|scope| {
///         // Handle events with mutable access
///         scope.handle_events_mut(DoubleHandler);
///     })
///     .build();
/// ```
#[derive(Debug)]
pub struct RustDisruptorBuilder {}

/// A builder stage that holds the data provider configuration.
///
/// This is the initial stage after creating a RustDisruptorBuilder,
/// allowing you to configure the wait strategy.
///
/// # Type Parameters
/// * `D` - The data provider type that implements DataProvider<T>
/// * `T` - The type of data elements stored in the ring buffer
pub struct WithDataProvider<D: DataProvider<T>, T>
where
    T: Send + Sync,
{
    data_provider: Arc<D>,
    _element: PhantomData<T>,
}

/// A builder stage that holds both data provider and wait strategy configurations.
///
/// This stage allows you to configure the sequencer type (single or multi-producer).
///
/// # Type Parameters
/// * `W` - The wait strategy type that implements WaitStrategy
/// * `D` - The data provider type that implements DataProvider<T>
/// * `T` - The type of data elements stored in the ring buffer
pub struct WithWaitStrategy<W: WaitStrategy, D: DataProvider<T>, T>
where
    T: Send + Sync,
{
    with_data_provider: WithDataProvider<D, T>,
    _wait_strategy: PhantomData<W>,
}

/// A builder stage that combines data provider, wait strategy, and sequencer configurations.
///
/// This stage allows you to set up event handlers and barriers.
///
/// # Type Parameters
/// * `S` - The sequencer type that implements Sequencer
/// * `W` - The wait strategy type that implements WaitStrategy
/// * `D` - The data provider type that implements DataProvider<T>
/// * `T` - The type of data elements stored in the ring buffer
pub struct WithSequencer<S: Sequencer, W: WaitStrategy, D: DataProvider<T>, T>
where
    T: Send + Sync,
{
    with_wait_strategy: WithWaitStrategy<W, D, T>,
    sequencer: S,
}

/// A scope for configuring barriers and event handlers.
///
/// This type provides methods for adding event handlers and managing sequences
/// within a specific barrier scope. The barrier ensures proper event processing
/// order and prevents data races.
///
/// # Type Parameters
/// * `'a` - The lifetime parameter for the scope
/// * `S` - The sequencer type that implements Sequencer
/// * `D` - The data provider type that implements DataProvider<T>
/// * `T` - The type of data elements stored in the ring buffer
pub struct BarrierScope<'a, S: Sequencer, D: DataProvider<T>, T> {
    sequencer: S,
    data_provider: Arc<D>,
    gating_sequences: Vec<Arc<AtomicSequence>>,
    cursors: Vec<Arc<AtomicSequence>>,
    event_handlers: Vec<Box<dyn Runnable + 'a>>,
    _element: PhantomData<T>,
}

/// The final builder stage that holds all configurations and event handlers.
///
/// This stage allows you to build the final ring buffer system or add more barriers.
///
/// # Type Parameters
/// * `'a` - The lifetime parameter for the event handlers
/// * `S` - The sequencer type that implements Sequencer
/// * `W` - The wait strategy type that implements WaitStrategy
/// * `D` - The data provider type that implements DataProvider<T>
/// * `T` - The type of data elements stored in the ring buffer
pub struct WithEventHandlers<'a, S: Sequencer, W: WaitStrategy, D: DataProvider<T>, T>
where
    T: Send + Sync,
{
    with_sequencer: WithSequencer<S, W, D, T>,
    event_handlers: Vec<Box<dyn Runnable + 'a>>,
    gating_sequences: Vec<Arc<AtomicSequence>>,
}

impl RustDisruptorBuilder {
    /// Creates a new builder with a custom data provider.
    ///
    /// This is typically used when you have a custom implementation of the
    /// DataProvider trait.
    ///
    /// # Type Parameters
    /// * `D` - The data provider type that implements DataProvider<T>
    /// * `T` - The type of data elements stored in the ring buffer
    ///
    /// # Arguments
    /// * `data_provider` - The custom data provider implementation
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

    /// Creates a new builder with a RingBuffer of specified capacity.
    ///
    /// # Type Parameters
    /// * `T` - The type of data elements stored in the ring buffer
    /// * `N` - A const generic parameter specifying the buffer size
    ///
    /// # Arguments
    /// * `capacity` - The buffer capacity, must be a power of two
    ///
    /// # Panics
    /// Panics if capacity is 0 or not a power of two
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
    /// Configures a custom wait strategy.
    ///
    /// # Type Parameters
    /// * `W` - The wait strategy type that implements WaitStrategy
    pub fn with_wait_strategy<W: WaitStrategy>(self) -> WithWaitStrategy<W, D, T> {
        WithWaitStrategy {
            with_data_provider: self,
            _wait_strategy: Default::default(),
        }
    }

    /// Configures a blocking wait strategy.
    ///
    /// This strategy is suitable for scenarios where low latency is not critical
    /// and you want to minimize CPU usage. It uses condition variables for thread
    /// synchronization.
    pub fn with_blocking_wait(self) -> WithWaitStrategy<BlockingWaitStrategy, D, T> {
        self.with_wait_strategy()
    }

    /// Configures a spin-loop wait strategy.
    ///
    /// This strategy is suitable for scenarios where low latency is critical
    /// and you are willing to use more CPU resources. It actively spins in a loop
    /// checking for sequence availability.
    pub fn with_spin_wait(self) -> WithWaitStrategy<SpinLoopWaitStrategy, D, T> {
        self.with_wait_strategy()
    }
}

impl<W: WaitStrategy, D: DataProvider<T>, T> WithWaitStrategy<W, D, T>
where
    T: Send + Sync,
{
    /// Configures a custom sequencer for the ring buffer.
    ///
    /// # Type Parameters
    /// * `S` - The sequencer type that implements Sequencer
    ///
    /// # Arguments
    /// * `sequencer` - A custom sequencer implementation
    pub fn with_sequencer<S: Sequencer>(self, sequencer: S) -> WithSequencer<S, W, D, T> {
        WithSequencer {
            with_wait_strategy: self,
            sequencer,
        }
    }

    /// Configures a single producer sequencer.
    ///
    /// This is optimized for scenarios where only one thread will be publishing
    /// events to the ring buffer. It provides better performance than the multi-producer
    /// sequencer in single-producer scenarios.
    pub fn with_single_producer(self) -> WithSequencer<SingleProducerSequencer<W>, W, D, T> {
        let buffer_size = self.with_data_provider.data_provider.buffer_size();
        self.with_sequencer(SingleProducerSequencer::new(buffer_size, W::new()))
    }

    /// Configures a multi-producer sequencer.
    ///
    /// This sequencer allows multiple threads to safely publish events to the ring buffer
    /// concurrently. While it has slightly higher overhead than the single-producer sequencer,
    /// it ensures thread safety in multi-producer scenarios.
    pub fn with_multi_producer(self) -> WithSequencer<MultiProducerSequencer<W>, W, D, T> {
        let buffer_size = self.with_data_provider.data_provider.buffer_size();
        self.with_sequencer(MultiProducerSequencer::new(buffer_size, W::new()))
    }
}

impl<'a, S: Sequencer + 'a, W: WaitStrategy, D: DataProvider<T> + 'a, T: Send + Sync + 'a>
WithSequencer<S, W, D, T>
{
    /// Configures event handlers within a barrier scope.
    ///
    /// This method provides a scope for configuring event handlers and managing
    /// sequences within a specific barrier. The barrier ensures proper event
    /// processing order and prevents data races.
    ///
    /// # Arguments
    /// * `f` - A closure that configures event handlers within the barrier scope
    ///
    /// # Returns
    /// Returns a WithEventHandlers builder stage that can be used to add more
    /// barriers or build the final ring buffer system
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
    /// Adds an event handler that processes events immutably.
    ///
    /// # Type Parameters
    /// * `E` - The event handler type, must implement EventHandler<T>
    ///
    /// # Arguments
    /// * `handler` - The event handler implementation
    pub fn handle_events<E>(&mut self, handler: E)
    where
        E: EventHandler<T> + Send + 'a,
    {
        self.handle_events_with(BatchEventProcessor::create(handler))
    }

    /// Adds an event handler that processes events with mutable access.
    ///
    /// # Type Parameters
    /// * `E` - The event handler type, must implement EventHandlerMut<T>
    ///
    /// # Arguments
    /// * `handler` - The event handler implementation
    pub fn handle_events_mut<E>(&mut self, handler: E)
    where
        E: EventHandlerMut<T> + Send + 'a,
    {
        self.handle_events_with(BatchEventProcessor::create_mut(handler))
    }

    /// Adds a custom event processor to the barrier scope.
    ///
    /// This is a lower-level method that allows you to add a custom event processor
    /// implementation directly, rather than using the higher-level handle_events methods.
    ///
    /// # Type Parameters
    /// * `E` - The event processor type, must implement EventProcessorMut<'a, T>
    ///
    /// # Arguments
    /// * `processor` - The event processor implementation
    pub fn handle_events_with<E: EventProcessorMut<'a, T>>(&mut self, processor: E) {
        self.cursors.push(processor.get_cursor());
        let barrier = self.sequencer.create_barrier(&self.gating_sequences);

        let runnable = processor.prepare(barrier, self.data_provider.clone());
        self.event_handlers.push(runnable);
    }

    /// Creates a nested barrier scope.
    ///
    /// This allows you to create a new barrier scope that inherits the current
    /// scope's configuration but can have its own additional event handlers and
    /// sequences.
    ///
    /// # Arguments
    /// * `f` - A closure that configures the nested barrier scope
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
    /// Creates a nested barrier scope.
    ///
    /// Similar to BarrierScope::with_barrier, this method allows you to create
    /// additional event handlers within a new barrier scope while maintaining
    /// the existing configuration.
    ///
    /// # Arguments
    /// * `f` - A closure that configures the nested barrier scope
    ///
    /// # Returns
    /// Returns self to allow for method chaining
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

    /// Builds the final ring buffer system using the default threaded executor.
    ///
    /// # Returns
    /// Returns a tuple containing:
    /// - An event processor executor that manages the event handlers
    /// - An event producer that can be used to publish events to the ring buffer
    pub fn build(
        self,
    ) -> (
        impl EventProcessorExecutor<'a>,
        impl EventProducer<'a, Item = T>,
    ) {
        self.build_with_executor::<ThreadedExecutor<'a>>()
    }

    /// Builds the final ring buffer system with a custom executor.
    ///
    /// This method allows you to specify a custom executor implementation for
    /// managing the event handlers, providing more control over how events are
    /// processed.
    ///
    /// # Type Parameters
    /// * `E` - The executor type, must implement EventProcessorExecutor<'a>
    ///
    /// # Returns
    /// Returns a tuple containing:
    /// - The custom event processor executor
    /// - An event producer that can be used to publish events to the ring buffer
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