use dcl_data_structures::ring_buffer::prelude::*;
use std::thread;
use std::time::{Duration, Instant};

// Define an immutable event handler for printing events
struct PrintHandler;
impl EventHandler<i32> for PrintHandler {
    fn handle_event(&self, event: &i32, sequence: u64, end_of_batch: bool) {
        println!("Received: {} at sequence {}", event, sequence);
        if end_of_batch {
            println!("End of batch at sequence {}", sequence);
        }
    }
}

// Define a mutable event handler that transforms events
struct MultiplyHandler {
    factor: i32,
}

impl EventHandlerMut<i32> for MultiplyHandler {
    fn handle_event(&mut self, event: &mut i32, sequence: u64, _end_of_batch: bool) {
        *event *= self.factor;
        println!("Multiplied event at sequence {}: new value = {}", sequence, event);
    }
}

fn main() {
    println!("\nRunning single producer example...");
    let start_time = Instant::now();

    // Create a ring buffer with single producer and blocking wait strategy
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<i32, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(|scope| {
            // First handle events immutably with PrintHandler
            scope.handle_events(PrintHandler);
            
            // Then handle events mutably with MultiplyHandler
            scope.handle_events_mut(MultiplyHandler { factor: 2 });
        })
        .build();

    // Start processing events in a separate thread
    let handle = executor.spawn();

    // Publish some events
    for i in 0..5 {
        let item = i;
        producer.write(std::iter::once(item), |slot, _, val| *slot = *val);
        thread::sleep(Duration::from_millis(100));
    }

    // Drain the producer and wait for processing to complete
    drop(producer);
    handle.join();
    
    let duration = start_time.elapsed();
    println!("Single producer example completed in {:?}", duration);
}
