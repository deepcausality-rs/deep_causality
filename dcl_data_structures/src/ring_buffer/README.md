# RingBuffer 

## Overview
The RingBuffer is a high-performance, lock-free data structure designed for efficient producer-consumer scenarios. 
This guide provides performance characteristics and recommendations based on extensive benchmarking.

## Performance Characteristics

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

1. **Batch Processing**
   - Always process events in batches when possible
   - Use the optimal batch size provided by the API
   - Adjust batch size based on your latency requirements

2. **Memory Layout**
   - RingBuffer is cache-line aligned for optimal performance
   - Keep hot data together to minimize cache misses
   - Consider CPU affinity for critical threads

3. **Error Handling**
   - Use appropriate error handling for buffer full/empty conditions
   - Implement backoff strategies for high contention scenarios

4. **Monitoring**
   - Monitor throughput and latency in production
   - Watch for signs of contention in multi-producer scenarios
   - Adjust batch sizes if performance degrades

## Example Usage

```rust
use dcl_data_structures::ring_buffer::prelude::*;

// Create a RingBuffer with optimal configuration
let buffer: RingBuffer<i64, 65536> = RingBuffer::new();

// Get the optimal batch size
let batch_size = RingBuffer::<i64, 65536>::optimal_batch_size();

// Use the optimal batch size for processing
let (start, end) = sequencer.next(batch_size);
// Process batch...
sequencer.publish(start, end);