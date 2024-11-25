use dcl_data_structures::ring_buffer::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// Define a consumer that processes and aggregates events from multiple producers
struct AggregateHandler {
    total_sum: i32,
    producer_counts: [i32; 3],
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
            "Consumer received: {} from producer {} at sequence {}. Running total: {}",
            value, producer_id, sequence, self.total_sum
        );
        
        if end_of_batch {
            println!("Batch ended at sequence {}. Producer event counts: {:?}", 
                sequence, self.producer_counts);
        }
    }
}

fn main() {
    println!("\nRunning multi-producer single consumer example...");
    let start_time = Instant::now();
    
    // Create a ring buffer with multiple producers and single consumer
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<(i32, usize), 1024>(1024)
        .with_blocking_wait()
        .with_multi_producer()  // Use multi-producer mode
        .with_barrier(|scope| {
            scope.handle_events_mut(AggregateHandler::new());
        })
        .build();

    // Start processing events
    let handle = executor.spawn();

    // Wrap the producer in an Arc for thread-safe sharing
    let producer = Arc::new(producer);

    // Create multiple producer threads
    let mut producer_handles = vec![];
    
    for producer_id in 0..3 {
        let producer = Arc::clone(&producer);
        let handle = thread::spawn(move || {
            // Each producer generates different values
            for i in 0..4 {
                let value = (producer_id + 1) * (i + 1);  // Unique pattern for each producer
                producer.write(std::iter::once((value as i32, producer_id)), |slot, _, val| *slot = *val);
                thread::sleep(Duration::from_millis(50 * (producer_id + 1) as u64));  // Different delays
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
    println!("Multi-producer single consumer example completed in {:?}", duration);
}
