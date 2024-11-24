use dcl_data_structures::ring_buffer::prelude::*;

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
