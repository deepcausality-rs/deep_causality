# RingBuffer

## Overview

The RingBuffer module is a high-performance, lock-free data structure implementation inspired by the LMAX Disruptor pattern. It provides a concurrent message-passing mechanism optimized for both high throughput and low latency scenarios. The implementation supports both single-producer and multi-producer configurations, with flexible event handling and customizable wait strategies.

### Key Features

- Lock-free implementation using atomic operations
- Support for both single-producer and multi-producer scenarios
- Flexible event handling with mutable and immutable handlers
- Customizable wait strategies (SpinLoop and Blocking)
- Batch processing capabilities for improved throughput
- DSL for easy configuration and setup
- Cache-line aligned for optimal performance

## Usage

### Basic Example

```rust
use dcl_data_structures::ring_buffer::prelude::*;

// Define an event handler
struct PrintHandler;
impl EventHandler<u64> for PrintHandler {
    fn handle_event(&self, event: &u64, sequence: u64, end_of_batch: bool) {
        println!("Received: {} at sequence {}", event, sequence);
    }
}

// Create a ring buffer with single producer
let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
    .with_blocking_wait()
    .with_single_producer()
    .with_barrier(|scope| {
        scope.handle_events(PrintHandler);
    })
    .build();

// Start processing events
executor.start();

// Publish events
producer.publish(42);
```

### Advanced Configuration

```rust
// Create a multi-producer ring buffer with custom wait strategy
let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<i32, 2048>(2048)
    .with_spin_wait()
    .with_multi_producer()
    .with_barrier(|scope| {
        // Add multiple handlers in sequence
        scope.handle_events(FirstHandler);
        scope.handle_events_mut(SecondHandler);
        
        // Create a nested barrier for parallel processing
        scope.with_barrier(|nested| {
            nested.handle_events(ParallelHandler1);
            nested.handle_events(ParallelHandler2);
        });
    })
    .build();
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

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
