#[cfg(test)]
mod tests {
    use dcl_data_structures::ring_buffer::prelude::*;

    #[test]
    fn test_immutable_event_handler() {
        struct TestHandler;
        impl EventHandler<u64> for TestHandler {
            fn handle_event(&self, event: &u64, _sequence: u64, _end_of_batch: bool) {
                println!("Handling event: {}", event);
            }
        }

        let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
            .with_blocking_wait()
            .with_single_producer()
            .with_barrier(|scope| {
                scope.handle_events(TestHandler);
            })
            .build();

        assert!(std::mem::size_of_val(&executor) > 0);
        assert!(std::mem::size_of_val(&producer) > 0);
    }

    #[test]
    fn test_mutable_event_handler() {
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
            .with_blocking_wait()
            .with_single_producer()
            .with_barrier(|scope| {
                scope.handle_events_mut(TestHandler { count: 0 });
            })
            .build();

        assert!(std::mem::size_of_val(&executor) > 0);
        assert!(std::mem::size_of_val(&producer) > 0);
    }

    #[test]
    fn test_custom_event_processor() {
        struct TestHandler {
            count: u64,
        }
        impl EventHandlerMut<u64> for TestHandler {
            fn handle_event(&mut self, event: &mut u64, _sequence: u64, _end_of_batch: bool) {
                self.count += 1;
                *event = self.count;
            }
        }

        let handler = TestHandler { count: 0 };
        let processor = BatchEventProcessor::create_mut(handler);

        let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
            .with_blocking_wait()
            .with_single_producer()
            .with_barrier(|scope| {
                scope.handle_events_with(processor);
            })
            .build();

        assert!(std::mem::size_of_val(&executor) > 0);
        assert!(std::mem::size_of_val(&producer) > 0);
    }

    #[test]
    fn test_multiple_handlers() {
        struct Handler1;
        impl EventHandler<u64> for Handler1 {
            fn handle_event(&self, event: &u64, _sequence: u64, _end_of_batch: bool) {
                println!("Handler1: {}", event);
            }
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

    #[test]
    fn test_handler_with_different_types() {
        struct FloatHandler;
        impl EventHandler<f64> for FloatHandler {
            fn handle_event(&self, event: &f64, _sequence: u64, _end_of_batch: bool) {
                println!("Handling float: {}", event);
            }
        }

        let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<f64, 1024>(1024)
            .with_blocking_wait()
            .with_single_producer()
            .with_barrier(|scope| {
                scope.handle_events(FloatHandler);
            })
            .build();

        assert!(std::mem::size_of_val(&executor) > 0);
        assert!(std::mem::size_of_val(&producer) > 0);
    }
}
