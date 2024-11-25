use dcl_data_structures::ring_buffer::prelude::*;
use std::thread;
use std::time::{Duration, Instant};

// Define event handlers for different consumers

// First consumer: Prints the original values
struct PrintHandler;
impl EventHandler<i32> for PrintHandler {
    fn handle_event(&self, event: &i32, sequence: u64, end_of_batch: bool) {
        println!("Consumer 1 received: {} at sequence {}", event, sequence);
        if end_of_batch {
            println!("Consumer 1 batch ended at sequence {}", sequence);
        }
    }
}

// Second consumer: Multiplies values by 2
struct MultiplyHandler;
impl EventHandlerMut<i32> for MultiplyHandler {
    fn handle_event(&mut self, event: &mut i32, sequence: u64, _end_of_batch: bool) {
        *event *= 2;
        println!("Consumer 2 multiplied: new value = {} at sequence {}", event, sequence);
    }
}

// Third consumer: Adds 10 to values
struct AddHandler;
impl EventHandlerMut<i32> for AddHandler {
    fn handle_event(&mut self, event: &mut i32, sequence: u64, _end_of_batch: bool) {
        *event += 10;
        println!("Consumer 3 added: new value = {} at sequence {}", event, sequence);
    }
}

fn main() {
    println!("\nRunning single producer with multiple consumers example...");
    let start_time = Instant::now();

    // Create a ring buffer with single producer and multiple consumers
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<i32, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        // First barrier: Print handler
        .with_barrier(|scope| {
            scope.handle_events(PrintHandler);
        })
        // Second barrier: Multiply handler
        .with_barrier(|scope| {
            scope.handle_events_mut(MultiplyHandler);
        })
        // Third barrier: Add handler
        .with_barrier(|scope| {
            scope.handle_events_mut(AddHandler);
        })
        .build();

    // Start processing events in separate threads
    let handle = executor.spawn();

    // Publish some events
    println!("Publishing events...");
    for i in 0..5 {
        let item = i + 1; // Start from 1 to make output clearer
        producer.write(std::iter::once(item), |slot, _, val| *slot = *val);
        thread::sleep(Duration::from_millis(100));
    }

    // Drain the producer and wait for processing to complete
    drop(producer);
    handle.join();
    
    let duration = start_time.elapsed();
    println!("Single producer multi-consumer example completed in {:?}", duration);
}
