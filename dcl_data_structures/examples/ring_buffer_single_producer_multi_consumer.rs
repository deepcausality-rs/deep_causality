//! Single Producer Multiple Consumer Ring Buffer Example
//!
//! This example shows how to process events through multiple consumers in parallel:
//! - One producer thread writing events
//! - Multiple consumers processing events in different ways
//!
//! Key points:
//! - Each consumer runs in its own barrier for parallel processing
//! - Consumers can be mixed (immutable and mutable)
//! - Events are processed by all consumers before moving to the next event
//! - Each consumer can maintain its own state

use dcl_data_structures::ring_buffer::prelude::*;
use std::thread;
use std::time::{Duration, Instant};

// First consumer: Immutable handler that prints events
// Shows how to implement basic event observation
struct PrintHandler;
impl EventHandler<i32> for PrintHandler {
    fn handle_event(&self, event: &i32, sequence: u64, end_of_batch: bool) {
        println!("Print handler received: {} at sequence {}", event, sequence);
        if end_of_batch {
            println!("Print handler batch ended at sequence {}", sequence);
        }
    }
}

// Second consumer: Mutable handler that multiplies events by 2
// Shows how to modify events in-place
struct MultiplyHandler;
impl EventHandlerMut<i32> for MultiplyHandler {
    fn handle_event(&mut self, event: &mut i32, sequence: u64, _end_of_batch: bool) {
        *event *= 2;
        println!(
            "Multiply handler: new value = {} at sequence {}",
            event, sequence
        );
    }
}

// Third consumer: Mutable handler that maintains running statistics
// Shows how to maintain state across events
struct StatsHandler {
    count: usize,
    sum: i32,
}

impl StatsHandler {
    fn new() -> Self {
        Self { count: 0, sum: 0 }
    }
}

impl EventHandlerMut<i32> for StatsHandler {
    fn handle_event(&mut self, event: &mut i32, sequence: u64, _end_of_batch: bool) {
        self.count += 1;
        self.sum += *event;
        println!(
            "Stats handler: count = {}, sum = {}, avg = {} at sequence {}",
            self.count,
            self.sum,
            self.sum as f64 / self.count as f64,
            sequence
        );
    }
}

fn main() {
    println!("\nRunning single producer with multiple consumers example...");
    let start_time = Instant::now();

    // STEP 1: Create the ring buffer with multiple consumers
    // Each consumer is in its own barrier for parallel processing
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<i32, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        // First consumer: Print events (immutable)
        .with_barrier(|scope| {
            scope.handle_events(PrintHandler);
        })
        // Second consumer: Multiply events (mutable)
        .with_barrier(|scope| {
            scope.handle_events_mut(MultiplyHandler);
        })
        // Third consumer: Track statistics (mutable with state)
        .with_barrier(|scope| {
            scope.handle_events_mut(StatsHandler::new());
        })
        .build();

    // STEP 2: Start event processing
    let handle = executor.spawn();

    // STEP 3: Publish events
    // Single producer can write events without synchronization
    for i in 0..5 {
        producer.write(std::iter::once(i + 1), |slot, _, val| *slot = *val);
        thread::sleep(Duration::from_millis(10)); // Simulated work
    }

    // STEP 4: Cleanup
    drop(producer);
    handle.join();

    // Performance Notes:
    // - Each consumer processes events in parallel
    // - Total processing time is roughly max(consumer_times) + overhead
    // - Real performance will be higher without prints and sleeps
    let duration = start_time.elapsed();
    println!(
        "Single producer multi-consumer example completed in {:?}",
        duration
    );
}
