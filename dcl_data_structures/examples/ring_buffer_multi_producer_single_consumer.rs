//! Multiple Producer Single Consumer Ring Buffer Example
//! 
//! This example demonstrates concurrent event production with:
//! - Multiple producer threads writing events concurrently
//! - One consumer aggregating and processing all events
//! 
//! Key points:
//! - Use `with_multi_producer()` for concurrent producers
//! - Wrap producer in Arc for thread-safe sharing
//! - Each producer can safely write events concurrently
//! - Consumer can track events from different producers

use dcl_data_structures::ring_buffer::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// Consumer that aggregates events from all producers
// Shows how to track events from different sources
struct AggregateHandler {
    total_sum: i32,
    producer_counts: [i32; 3],  // Track events from each producer
}

impl AggregateHandler {
    fn new() -> Self {
        Self {
            total_sum: 0,
            producer_counts: [0; 3],
        }
    }
}

impl EventHandlerMut<(i32, usize)> for AggregateHandler {
    fn handle_event(&mut self, event: &mut (i32, usize), sequence: u64, end_of_batch: bool) {
        let (value, producer_id) = *event;
        self.total_sum += value;
        self.producer_counts[producer_id] += 1;
        
        println!(
            "Received value {} from producer {} at sequence {}. Running total: {}",
            value, producer_id, sequence, self.total_sum
        );
        
        if end_of_batch {
            println!(
                "Batch ended at sequence {}. Events per producer: {:?}", 
                sequence, self.producer_counts
            );
        }
    }
}

fn main() {
    println!("\nRunning multi-producer single consumer example...");
    let start_time = Instant::now();
    
    // STEP 1: Create the ring buffer
    // Use multi-producer mode for concurrent event publishing
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<(i32, usize), 1024>(1024)
        .with_blocking_wait()
        .with_multi_producer()
        .with_barrier(|scope| {
            scope.handle_events_mut(AggregateHandler::new());
        })
        .build();

    // STEP 2: Start event processing
    let handle = executor.spawn();

    // STEP 3: Share producer across threads
    let producer = Arc::new(producer);

    // STEP 4: Create producer threads
    let mut producer_handles = vec![];
    
    // Launch multiple producers
    for producer_id in 0..3 {
        let producer = Arc::clone(&producer);
        let handle = thread::spawn(move || {
            // Each producer generates unique values
            for i in 0..4 {
                let value = (producer_id + 1) * (i + 1);
                producer.write(
                    std::iter::once((value as i32, producer_id)),
                    |slot, _, val| *slot = *val
                );
                thread::sleep(Duration::from_millis(10));  // Simulated work
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
    // - Multiple producers can write concurrently
    // - Thread synchronization adds some overhead
    // - Real performance scales with number of producers
    let duration = start_time.elapsed();
    println!("Multi-producer single consumer example completed in {:?}", duration);
}
