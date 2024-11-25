use dcl_data_structures::ring_buffer::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// First consumer: Prints and counts events from each producer
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
            "Consumer 1 (Print) received: {} from producer {} at sequence {}",
            value, producer_id, sequence
        );
        if end_of_batch {
            println!("Consumer 1 batch ended at sequence {}", sequence);
        }
    }
}

// Second consumer: Multiplies values by 2
struct MultiplyHandler;

impl EventHandlerMut<(i32, usize)> for MultiplyHandler {
    fn handle_event(&mut self, event: &mut (i32, usize), sequence: u64, _end_of_batch: bool) {
        let (value, producer_id) = *event;
        *event = (value * 2, producer_id);
        println!(
            "Consumer 2 (Multiply) processed: new value = {} at sequence {}",
            event.0, sequence
        );
    }
}

// Third consumer: Adds 10 to values and maintains running total
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
            "Consumer 3 (Add) processed: new value = {}, running total = {} at sequence {}",
            event.0, self.running_total, sequence
        );
    }
}

fn main() {
    println!("\nRunning multi-producer multi-consumer example...");
    let start_time = Instant::now();
    
    // Create a ring buffer with multiple producers and multiple consumers
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<(i32, usize), 1024>(1024)
        .with_blocking_wait()
        .with_multi_producer()
        // First barrier: Print handler (immutable)
        .with_barrier(|scope| {
            scope.handle_events(PrintHandler::new());
        })
        // Second barrier: Multiply handler (mutable)
        .with_barrier(|scope| {
            scope.handle_events_mut(MultiplyHandler);
        })
        // Third barrier: Add handler (mutable)
        .with_barrier(|scope| {
            scope.handle_events_mut(AddHandler::new());
        })
        .build();

    // Start processing events
    let handle = executor.spawn();

    // Wrap the producer in an Arc for thread-safe sharing
    let producer = Arc::new(producer);

    // Create multiple producer threads
    let mut producer_handles = vec![];
    
    // Launch three producers
    for producer_id in 0..3 {
        let producer = Arc::clone(&producer);
        let handle = thread::spawn(move || {
            // Each producer generates different values with unique patterns
            for i in 0..4 {
                let value = (producer_id + 1) * (i + 1);  // Unique pattern for each producer
                producer.write(std::iter::once((value as i32, producer_id)), |slot, _, val| *slot = *val);
                
                // Different delays for each producer to simulate varying processing times
                thread::sleep(Duration::from_millis(50 * (producer_id + 1) as u64));
            }
            println!("Producer {} finished", producer_id);
        });
        producer_handles.push(handle);
    }

    // Wait for all producers to finish
    for handle in producer_handles {
        handle.join().unwrap();
    }

    // Clean up and wait for processing to complete
    drop(producer);
    handle.join();
    
    let duration = start_time.elapsed();
    println!("Multi-producer multi-consumer example completed in {:?}", duration);
}
