use dcl_data_structures::ring_buffer::prelude::*;
use std::sync::Arc;

#[test]
fn test_single_executor() {
    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(|_scope| {})
        .build();

    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_executor_with_multiple_handlers() {
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

    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_executor_with_custom_handler() {
    struct CustomHandler {
        count: Arc<AtomicSequence>,
    }

    impl EventHandler<u64> for CustomHandler {
        fn handle_event(&self, event: &u64, sequence: u64, _end_of_batch: bool) {
            self.count.set(sequence);
            println!("Handling event {} at sequence {}", event, sequence);
        }
    }

    let counter = Arc::new(AtomicSequence::default());
    let handler = CustomHandler {
        count: counter.clone(),
    };

    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(|scope| {
            scope.handle_events(handler);
        })
        .build();

    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_executor_with_mutable_handler() {
    struct MutableHandler {
        count: u64,
    }

    impl EventHandlerMut<u64> for MutableHandler {
        fn handle_event(&mut self, event: &mut u64, _sequence: u64, _end_of_batch: bool) {
            self.count += 1;
            *event = self.count;
        }
    }

    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(|scope| {
            scope.handle_events_mut(MutableHandler { count: 0 });
        })
        .build();

    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}

#[test]
fn test_executor_with_sequential_barriers() {
    struct Handler1;
    impl EventHandler<u64> for Handler1 {
        fn handle_event(&self, _event: &u64, _sequence: u64, _end_of_batch: bool) {}
    }

    struct Handler2 {
        count: u64,
    }
    impl EventHandlerMut<u64> for Handler2 {
        fn handle_event(&mut self, event: &mut u64, _sequence: u64, _end_of_batch: bool) {
            self.count += 1;
            *event = self.count;
        }
    }

    let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
        .with_blocking_wait()
        .with_single_producer()
        .with_barrier(|scope| {
            scope.handle_events(Handler1);
        })
        .with_barrier(|scope| {
            scope.handle_events_mut(Handler2 { count: 0 });
        })
        .build();

    assert!(std::mem::size_of_val(&executor) > 0);
    assert!(std::mem::size_of_val(&producer) > 0);
}
