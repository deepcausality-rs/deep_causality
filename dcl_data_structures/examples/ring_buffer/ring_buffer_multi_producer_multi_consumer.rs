//! Multiple Producer Multiple Consumer Ring Buffer Example
//!
//! This example demonstrates the most complex ring buffer configuration:
//! - Multiple producers writing events concurrently
//! - Multiple consumers processing events in parallel
//!
//! Key points:
//! - Combines features of multi-producer and multi-consumer patterns
//! - Each producer writes independently
//! - Consumers process in parallel but maintain ordering
//! - Great for high-throughput event processing pipelines

use dcl_data_structures::ring_buffer::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// First consumer: Tracks and reports events from each producer
// Shows how to maintain producer-specific state
#[allow(dead_code)]
struct PrintHandler {
    producer_counts: [i32; 3],
}

impl PrintHandler {
    fn new() -> Self {
        Self {
            producer_counts: [0; 3],
        }
    }
}

impl EventHandler<(i32, usize)> for PrintHandler {
    fn handle_event(&self, event: &(i32, usize), sequence: u64, end_of_batch: bool) {
        let (value, producer_id) = *event;
        println!(
            "Print handler: value {} from producer {} at sequence {}",
            value, producer_id, sequence
        );
        if end_of_batch {
            println!("Print handler batch ended at sequence {}", sequence);
        }
    }
}

// Second consumer: Transforms events by multiplication
// Shows how to modify events in-place
struct MultiplyHandler;

impl EventHandlerMut<(i32, usize)> for MultiplyHandler {
    fn handle_event(&mut self, event: &mut (i32, usize), sequence: u64, _end_of_batch: bool) {
        let (value, producer_id) = *event;
        *event = (value * 2, producer_id);
        println!(
            "Multiply handler: new value = {} at sequence {}",
            event.0, sequence
        );
    }
}

// Third consumer: Maintains running statistics
// Shows how to aggregate across all producers
struct AddHandler {
    running_total: i32,
}

impl AddHandler {
    fn new() -> Self {
        Self { running_total: 0 }
    }
}

impl EventHandlerMut<(i32, usize)> for AddHandler {
    fn handle_event(&mut self, event: &mut (i32, usize), sequence: u64, _end_of_batch: bool) {
        let (value, producer_id) = *event;
        *event = (value + 10, producer_id);
        self.running_total += event.0;
        println!(
            "Add handler: value = {}, running total = {} at sequence {}",
            event.0, self.running_total, sequence
        );
    }
}

fn main() {
    println!("\nRunning multi-producer multi-consumer example...");
    let start_time = Instant::now();

    // STEP 1: Create ring buffer with multiple barriers
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<(i32, usize), 1024>(1024)
        .with_blocking_wait()
        .with_multi_producer()
        // First barrier: Track events (immutable)
        .with_barrier(|scope| {
            scope.handle_events(PrintHandler::new());
        })
        // Second barrier: Transform events (mutable)
        .with_barrier(|scope| {
            scope.handle_events_mut(MultiplyHandler);
        })
        // Third barrier: Aggregate results (mutable)
        .with_barrier(|scope| {
            scope.handle_events_mut(AddHandler::new());
        })
        .build();

    // STEP 2: Start event processing
    let handle = executor.spawn();

    // STEP 3: Share producer across threads
    let producer = Arc::new(producer);

    // STEP 4: Launch producer threads
    let mut producer_handles = vec![];

    for producer_id in 0..3 {
        let producer = Arc::clone(&producer);
        let handle = thread::spawn(move || {
            // Each producer generates unique patterns
            for i in 0..4 {
                let value = (producer_id + 1) * (i + 1);
                producer.write(
                    std::iter::once((value as i32, producer_id)),
                    |slot, _, val| *slot = *val,
                );
                thread::sleep(Duration::from_millis(10)); // Simulated work
            }
            println!("Producer {} finished", producer_id);
        });
        producer_handles.push(handle);
    }

    // STEP 5: Wait for producers and cleanup
    for handle in producer_handles {
        handle.join().unwrap();
    }
    drop(producer);
    handle.join();

    // Performance Notes:
    // - Producers write concurrently
    // - Consumers process in parallel
    // - Total throughput = min(producer_rate, consumer_rate)
    // - Real performance much higher without prints/sleeps
    let duration = start_time.elapsed();
    println!(
        "Multi-producer multi-consumer example completed in {:?}",
        duration
    );
}
