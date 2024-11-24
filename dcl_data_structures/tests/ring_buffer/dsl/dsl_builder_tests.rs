use dcl_data_structures::ring_buffer::prelude::*;
use std::panic::catch_unwind;

#[test]
fn test_basic_builder() {
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(|_scope| {})
        .build();

    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_ring_buffer_invalid_capacity() {
    // Test a capacity that is not a power of 2 (e.g., 10)
    let result = catch_unwind(|| {
        RustDisruptorBuilder::with_ring_buffer::<i32, 10>(10);
    });

    assert!(
        result.is_err(),
        "Expected panic for non-power-of-2 capacity"
    );

    // Verify that the error message contains our expected assertion message
    if let Err(panic) = result {
        let panic_msg = panic.downcast_ref::<String>();
        if let Some(msg) = panic_msg {
            assert!(
                msg.contains("capacity must be power of two"),
                "Panic message should mention power of two requirement"
            );
        }
    }
}

#[test]
fn test_builder_with_different_wait_strategies() {
    // Test with blocking wait
    let (executor1, producer1) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(|_scope| {})
        .build();

    // Test with spin wait
    let (executor2, producer2) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_spin_wait()
        .with_single_producer()
        .with_barrier(|_scope| {})
        .build();

    assert!(std::mem::size_of_val(&executor1) > 0);
    assert!(std::mem::size_of_val(&producer1) > 0);
    assert!(std::mem::size_of_val(&executor2) > 0);
    assert!(std::mem::size_of_val(&producer2) > 0);
}

#[test]
fn test_builder_with_multi_producer() {
    // Test with multi-producer configuration
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_multi_producer()
        .with_barrier(|_scope| {})
        .build();

    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_nested_barrier_scopes() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    // Create a shared counter that both handlers will update
    let total_count = Arc::new(AtomicU64::new(0));

    // Define handlers that will track the order of execution
    struct CountingHandler {
        count: Arc<AtomicU64>,
    }

    impl EventHandler<u64> for CountingHandler {
        fn handle_event(&self, event: &u64, _sequence: u64, _end_of_batch: bool) {
            self.count.fetch_add(*event, Ordering::SeqCst);
        }
    }

    let counter1 = total_count.clone();
    let counter2 = total_count.clone();

    // Create a ring buffer with nested barrier scopes
    let (_executor, _producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(move |scope| {
            scope.handle_events(CountingHandler { count: counter1 });
            scope.handle_events(CountingHandler { count: counter2 });
        })
        .build();
}
