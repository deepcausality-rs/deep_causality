//! Single Producer Single Consumer Ring Buffer Example
//!
//! This example demonstrates the simplest ring buffer configuration with:
//! - One producer thread writing events
//! - Two consumers in sequence (one immutable, one mutable)
//!
//! Key points:
//! - Use `with_single_producer()` for single producer scenarios (more efficient than multi-producer)
//! - Chain handlers using barriers for sequential processing
//! - Events flow through handlers in the order they're added
//! - Handlers can be immutable (EventHandler) or mutable (EventHandlerMut)

use dcl_data_structures::ring_buffer::prelude::*;
use std::thread;
use std::time::{Duration, Instant};

// First handler: Immutable handler that just prints events
// Implement EventHandler for read-only access to events
struct PrintHandler;
impl EventHandler<i32> for PrintHandler {
    fn handle_event(&self, event: &i32, sequence: u64, end_of_batch: bool) {
        println!("Received: {} at sequence {}", event, sequence);
        if end_of_batch {
            println!("End of batch at sequence {}", sequence);
        }
    }
}

// Second handler: Mutable handler that modifies events
// Implement EventHandlerMut to modify events in-place
struct MultiplyHandler {
    factor: i32,
}
impl EventHandlerMut<i32> for MultiplyHandler {
    fn handle_event(&mut self, event: &mut i32, sequence: u64, _end_of_batch: bool) {
        *event *= self.factor;
        println!(
            "Multiplied event at sequence {}: new value = {}",
            sequence, event
        );
    }
}

fn main() {
    println!("\nRunning single producer example...");
    let start_time = Instant::now();

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
        .with_barrier(|scope| {
            scope.handle_events_mut(MultiplyHandler { factor: 2 });
        })
        .build();

    // STEP 2: Start event processing in a separate thread
    let handle = executor.spawn();

    // STEP 3: Publish events
    // The producer can be used from a single thread without synchronization
    for i in 0..5 {
        producer.write(std::iter::once(i + 1), |slot, _, val| *slot = *val);
        thread::sleep(Duration::from_millis(10)); // Simulated work
    }

    // STEP 4: Cleanup
    // Drop the producer and wait for all events to be processed
    drop(producer);
    handle.join();

    // Performance Notes:
    // - 10ms sleep time per iteration simulates work
    // - With 4 iterations, total sleep time is 40ms
    // - Remaining time is thread management and I/O overhead
    // - Real performance will be much higher without sleeps and prints
    let duration = start_time.elapsed();
    println!("Single producer example completed in {:?}", duration);
}
