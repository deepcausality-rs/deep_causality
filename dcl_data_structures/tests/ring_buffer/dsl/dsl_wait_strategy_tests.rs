use dcl_data_structures::ring_buffer::prelude::*;

#[test]
fn test_blocking_wait_strategy() {
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(|_scope| {})
        .build();

    // The builder should successfully create an executor and producer with blocking wait
    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_spin_wait_strategy() {
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_spin_wait()
        .with_single_producer()
        .with_barrier(|_scope| {})
        .build();

    // The builder should successfully create an executor and producer with spin wait
    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_wait_strategy_with_multiple_handlers() {
    struct TestHandler;
    impl EventHandler<u64> for TestHandler {
        fn handle_event(&self, _event: &u64, _sequence: u64, _end_of_batch: bool) {}
    }

    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(|scope| {
            scope.handle_events(TestHandler);
            scope.handle_events(TestHandler);
        })
        .build();

    // Should successfully create executor and producer with multiple handlers
    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_wait_strategy_with_sequential_barriers() {
    struct TestHandler;
    impl EventHandler<u64> for TestHandler {
        fn handle_event(&self, _event: &u64, _sequence: u64, _end_of_batch: bool) {}
    }

    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_spin_wait()
        .with_single_producer()
        .with_barrier(|scope| {
            scope.handle_events(TestHandler);
        })
        .with_barrier(|scope| {
            scope.handle_events(TestHandler);
        })
        .build();

    // Should successfully create executor and producer with sequential barriers
    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_wait_strategy_with_mutable_handler() {
    struct TestHandler {
        count: u64,
    }
    impl EventHandlerMut<u64> for TestHandler {
        fn handle_event(&mut self, event: &mut u64, _sequence: u64, _end_of_batch: bool) {
            self.count += 1;
            *event = self.count;
        }
    }

    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_spin_wait()
        .with_single_producer()
        .with_barrier(|scope| {
            scope.handle_events_mut(TestHandler { count: 0 });
        })
        .build();

    // Should successfully create executor and producer with mutable handler
    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}
