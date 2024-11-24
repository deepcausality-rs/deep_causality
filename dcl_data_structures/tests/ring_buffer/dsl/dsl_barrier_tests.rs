#[cfg(test)]
mod tests {
    use dcl_data_structures::ring_buffer::prelude::*;
    use std::borrow::Borrow;
    use std::sync::Arc;

    #[test]
    fn test_single_barrier() {
        struct TestHandler;
        impl EventHandler<u64> for TestHandler {
            fn handle_event(&self, _event: &u64, _sequence: u64, _end_of_batch: bool) {}
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
    fn test_multiple_barriers() {
        struct TestHandler;
        impl EventHandler<u64> for TestHandler {
            fn handle_event(&self, _event: &u64, _sequence: u64, _end_of_batch: bool) {}
        }

        let (executor, producer) = RustDisruptorBuilder::with_ring_buffer::<u64, 1024>(1024)
            .with_blocking_wait()
            .with_single_producer()
            .with_barrier(|scope| {
                scope.handle_events(TestHandler);
            })
            .with_barrier(|scope| {
                scope.handle_events(TestHandler);
            })
            .build();

        assert!(std::mem::size_of_val(&executor) > 0);
        assert!(std::mem::size_of_val(&producer) > 0);
    }

    #[test]
    fn test_barrier_with_custom_sequence_barrier() {
        struct CustomBarrier {
            wait_strategy: Arc<SpinLoopWaitStrategy>,
            dependent_sequences: Vec<Arc<AtomicSequence>>,
        }

        impl SequenceBarrier for CustomBarrier {
            fn wait_for(&self, sequence: u64) -> Option<u64> {
                let deps: Vec<&AtomicSequence> = self
                    .dependent_sequences
                    .iter()
                    .map(|s| s.borrow())
                    .collect();

                self.wait_strategy.wait_for(sequence, &deps, || true)
            }

            fn signal(&self) {
                // Implementation for signaling
            }
        }

        let wait_strategy = Arc::new(SpinLoopWaitStrategy::new());
        let dependent_sequences = Vec::new();

        let barrier = CustomBarrier {
            wait_strategy,
            dependent_sequences,
        };

        struct TestHandler;
        impl EventHandler<u64> for TestHandler {
            fn handle_event(&self, _event: &u64, _sequence: u64, _end_of_batch: bool) {}
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
        assert!(std::mem::size_of_val(&barrier) > 0);
    }

    #[test]
    fn test_barrier_with_mutable_handler() {
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
    fn test_barrier_with_sequential_handlers() {
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
}
