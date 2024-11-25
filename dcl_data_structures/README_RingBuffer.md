# RingBuffer


The RingBuffer module is a high-performance, lock-free data structure implementation inspired by the LMAX Disruptor pattern. It provides a concurrent message-passing mechanism optimized for both high throughput and low latency scenarios. The implementation supports both single-producer and multi-producer configurations, with flexible event handling and customizable wait strategies.

### Supported configurartions

- Single producer / single consumer
- Single producer / muliple consumer
- Multi producer / single consumer
- Multi producer / multi consumer

### Key Features

- Blocking and spinning wait strategies
- Event batching for improved throughput
- Thread-safe concurrent operations
- Zero-allocation event handling
- Lock-free implementation using atomic operations
- Flexible event handling with mutable and immutable handlers
- Customizable wait strategies (SpinLoop and Blocking)
- Batch processing capabilities for improved throughput
- DSL for easy configuration and setup
- Cache-line aligned for optimal performance

## Build and Test

Build all examples:
```bash
cargo build --examples
```

Run tests:
```bash
cargo test ring_buffer
```

## Usage

```rust
use dcl_data_structures::ring_buffer::prelude::*;
use std::thread;
use std::time::{Duration, Instant};

// First handler: Immutable handler that just prints events
// Implement EventHandler for read-only access to events. 
// For mutable handlers, implement the EventHandlerMut trait.
struct PrintHandler;
impl EventHandler<i32> for PrintHandler {
    fn handle_event(&self, event: &i32, sequence: u64, end_of_batch: bool) {
        println!("Received: {} at sequence {}", event, sequence);
        if end_of_batch {
            println!("End of batch at sequence {}", sequence);
        }
    }
}

    // STEP 1: Create the ring buffer
    // - Specify the event type (i32) and buffer size (1024)
    // - Use blocking wait strategy for simplicity
    // - Configure for single producer
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<i32, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        // Add handlers in sequence - events flow through them in this order
        .with_barrier(|scope| {
            scope.handle_events(PrintHandler);
        })
        .build();

    // STEP 2: Start event processing in a separate thread
    let handle = executor.spawn();

    // STEP 3: Publish events
    // The producer can be used from a single thread without synchronization
    for i in 0..5 {
        producer.write(std::iter::once(i + 1), |slot, _, val| *slot = *val);
        thread::sleep(Duration::from_millis(10));  // Simulated work
    }

    // STEP 4: Cleanup
    // Drop the producer and wait for all events to be processed
    drop(producer);
    handle.join();
```


## Examples

The following examples demonstrate different ring buffer configurations, from simple to complex.
Please note, that all examples include timing measurements, but the actual performance will be substantially higher than shown in examples. 

- For best performance:
  - Remove sleep delays
  - Disable debug printing
  - Use release mode builds
  - Consider using spinning wait strategy for low latency

### 1. Single Producer Single Consumer
[`examples/ring_buffer_single_producer_single_consumer.rs`](examples/ring_buffer_single_producer_single_consumer.rs)

Demonstrates the simplest ring buffer configuration:
- One producer thread writing events
- Two consumers in sequence (one immutable, one mutable)
- Shows basic event handling and transformation

Run with:
```bash
cargo run --example ring_buffer_single_producer_single_consumer
```

### 2. Single Producer Multiple Consumers
[`examples/ring_buffer_single_producer_multi_consumer.rs`](examples/ring_buffer_single_producer_multi_consumer.rs)

Shows parallel event processing:
- One producer thread
- Multiple consumers processing events concurrently
- Demonstrates state tracking and event transformation
- Shows how to maintain consumer-specific state

Run with:
```bash
cargo run --example ring_buffer_single_producer_multi_consumer
```

### 3. Multiple Producers Single Consumer
[`examples/ring_buffer_multi_producer_single_consumer.rs`](examples/ring_buffer_multi_producer_single_consumer.rs)

Illustrates concurrent event production:
- Multiple producer threads writing concurrently
- One consumer aggregating all events
- Shows how to track events from different producers
- Demonstrates thread-safe producer sharing

Run with:
```bash
cargo run --example ring_buffer_multi_producer_single_consumer
```

### 4. Multiple Producers Multiple Consumers
[`examples/ring_buffer_multi_producer_multi_consumer.rs`](examples/ring_buffer_multi_producer_multi_consumer.rs)

The most complex configuration, combining:
- Multiple producers writing concurrently
- Multiple consumers processing in parallel
- Full event pipeline with transformation and aggregation
- Perfect for high-throughput event processing

Run with:
```bash
cargo run --example ring_buffer_multi_producer_multi_consumer
```

## Implementation

The RingBuffer implementation consists of several key components:

### Core Components

1. **RingBuffer**: The central data structure that stores events in a circular buffer.
   - Uses atomic operations for thread-safe access
   - Implements cache-line padding to prevent false sharing
   - Supports power-of-2 sizes for optimal indexing

2. **Sequencers**: Manage sequence numbers for producers and consumers
   - SingleProducerSequencer: Optimized for single-thread publishing
   - MultiProducerSequencer: Ensures thread-safe publishing from multiple threads

3. **Wait Strategies**: Control how consumers wait for new events
   - SpinLoopWaitStrategy: Active spinning for lowest latency
   - BlockingWaitStrategy: Condition variables for power efficiency

4. **Event Processors**: Handle event processing and batching
   - BatchEventProcessor: Processes events in batches for improved throughput
   - Support for both mutable and immutable event handlers

### Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Producer  │ -> │  RingBuffer │ -> │  Consumer   │
└─────────────┘    └─────────────┘    └─────────────┘
       ↑                  ↑                  ↑
       │                  │                  │
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Sequencer  │    │    Data     │    │    Event    │
│             │ -> │  Provider   │ -> │  Processor  │
└─────────────┘    └─────────────┘    └─────────────┘
```

## Performance

### Single Producer Performance
| Batch Size | Throughput      | Latency    |
|------------|-----------------|------------|
| 1          | 220.47 Melem/s  | 4.54 ms   |
| 10         | 1.65 Gelem/s    | 604.88 µs |
| 50         | 1.67 Gelem/s    | 597.67 µs |
| 100        | 1.68 Gelem/s    | 596.12 µs |

### Multi Producer Performance
| Batch Size | Throughput     | Latency    |
|------------|----------------|------------|
| 1          | 19.24 Melem/s  | 51.97 ms  |
| 10         | 162.09 Melem/s | 6.17 ms   |
| 50         | 273.06 Melem/s | 3.66 ms   |
| 100        | 332.22 Melem/s | 3.01 ms   |

### Hardware & OS
- Architecture: ARM64 (Apple Silicon, M3 Max)
- OS: macOS Darwin 24.1.0 (Seqoia 15.1)
- Kernel: XNU 11215.41.3~2
- Machine: MacBook Pro (T6031)

## Optimization Guidelines

### 1. Batch Size Selection
- For single-producer: Use batch sizes between 50-100 for optimal throughput
- For multi-producer: Use batch size of 100 for best balance of throughput and latency
- Use `RingBuffer::optimal_batch_size()` for a good default value

### 2. Wait Strategy Selection
- SpinLoop: Best for low-latency requirements
- Blocking: Better for power efficiency
- Choose based on your specific requirements:
  - Low latency: Use SpinLoop
  - Power efficiency: Use Blocking
  - Mixed workload: Consider SpinLoop with small batch sizes

### 3. Buffer Size Configuration
- Always use power of 2 sizes
- Default size (65536) works well for most cases
- Adjust based on your memory constraints and usage patterns

## Best Practices

### 1. Batch Processing
- Always process events in batches when possible
- Use the optimal batch size provided by the API
- Adjust batch size based on your latency requirements

### 2. Memory Layout
- RingBuffer is cache-line aligned for optimal performance
- Keep hot data together to minimize cache misses
- Consider CPU affinity for critical threads

### 3. Error Handling
- Use appropriate error handling for buffer full/empty conditions
- Implement backoff strategies for high contention scenarios
- Consider using Result types for error propagation

### 4. Monitoring
- Monitor throughput and latency in production
- Watch for signs of contention in multi-producer scenarios
- Adjust batch sizes if performance degrades

### 5. Thread Management
- Assign appropriate thread priorities
- Consider using dedicated threads for critical producers/consumers
- Implement proper shutdown procedures

## Common Pitfalls

1. **Non-Power-of-2 Buffer Sizes**
   - Always use power of 2 sizes to ensure optimal performance
   - Incorrect sizes will cause assertion failures

2. **Blocking in Event Handlers**
   - Avoid blocking operations in event handlers
   - Use async processing for I/O operations

3. **Insufficient Batch Sizes**
   - Too small batch sizes can limit throughput
   - Too large batch sizes can increase latency
   - Use performance metrics to find the right balance

4. **Memory Barriers**
   - Be aware of memory ordering requirements
   - Use appropriate atomic operations

## License

This project is licensed under the MIT License - see the LICENSE file for details.
