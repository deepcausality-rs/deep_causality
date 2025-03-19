use dcl_data_structures::ring_buffer::prelude::*;
use std::sync::Arc;
use std::thread;

#[test]
fn test_new_ringbuffer() {
    let rb: RingBuffer<i32, 16> = RingBuffer::new();
    assert_eq!(rb.buffer_size(), 16);
}

#[test]
#[should_panic(expected = "capacity must be power of two")]
fn test_new_ringbuffer_non_power_of_two() {
    let _rb: RingBuffer<i32, 15> = RingBuffer::new(); // 15 is not a power of 2
}

#[test]
#[should_panic(expected = "capacity must be power of two")]
fn test_new_ringbuffer_zero_capacity() {
    let _rb: RingBuffer<i32, 0> = RingBuffer::new();
}

#[test]
fn test_default_implementation() {
    let rb: RingBuffer<i32, 16> = RingBuffer::default();
    assert_eq!(rb.buffer_size(), 16);
}

#[test]
fn test_optimal_batch_size() {
    assert_eq!(RingBuffer::<i32, 16>::optimal_batch_size(), 80);
    assert_eq!(RingBuffer::<bool, 64>::optimal_batch_size(), 80);
    assert_eq!(RingBuffer::<u8, 128>::optimal_batch_size(), 80);
}

#[test]
fn test_buffer_size() {
    let rb8: RingBuffer<i32, 8> = RingBuffer::new();
    let rb16: RingBuffer<i32, 16> = RingBuffer::new();
    let rb32: RingBuffer<i32, 32> = RingBuffer::new();

    assert_eq!(rb8.buffer_size(), 8);
    assert_eq!(rb16.buffer_size(), 16);
    assert_eq!(rb32.buffer_size(), 32);
}

#[test]
fn test_get_and_set() {
    let rb: RingBuffer<i32, 16> = RingBuffer::new();

    // Test writing and reading at different positions
    unsafe {
        *rb.get_mut(0) = 42;
        assert_eq!(*rb.get(0), 42);

        *rb.get_mut(15) = 100;
        assert_eq!(*rb.get(15), 100);
    }
}

#[test]
fn test_wrapping_behavior() {
    let rb: RingBuffer<i32, 4> = RingBuffer::new();

    // Write values that wrap around the buffer
    unsafe {
        *rb.get_mut(0) = 1;
        *rb.get_mut(4) = 2; // Should wrap to index 0
        *rb.get_mut(8) = 3; // Should wrap to index 0

        // Verify wrapping behavior
        assert_eq!(*rb.get(8), 3); // Should read from index 0
        assert_eq!(*rb.get(0), 3); // Same physical location
    }
}

#[test]
fn test_different_types() {
    // Test with various types
    let rb_i32: RingBuffer<i32, 16> = RingBuffer::new();
    let rb_bool: RingBuffer<bool, 16> = RingBuffer::new();
    let rb_f64: RingBuffer<f64, 16> = RingBuffer::new();

    unsafe {
        // Test i32
        *rb_i32.get_mut(0) = 42;
        assert_eq!(*rb_i32.get(0), 42);

        // Test bool
        *rb_bool.get_mut(0) = true;
        assert!(*rb_bool.get(0));

        // Test f64
        *rb_f64.get_mut(0) = std::f64::consts::PI;
        assert!(((*rb_f64.get(0) - std::f64::consts::PI).abs() < f64::EPSILON));
    }
}

#[test]
fn test_thread_safety() {
    let rb = Arc::new(RingBuffer::<i32, 16>::new());
    let mut handles = vec![];

    // Spawn multiple threads to write and read
    for i in 0..4 {
        let rb_clone = Arc::clone(&rb);
        let handle = thread::spawn(move || {
            unsafe {
                *rb_clone.get_mut(i) = i as i32;
                thread::yield_now(); // Increase chance of race conditions
                assert_eq!(*rb_clone.get(i), i as i32);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_cache_alignment() {
    use std::mem;
    // Verify the struct is cache-line aligned (64 bytes)
    assert_eq!(mem::align_of::<RingBuffer<i32, 16>>(), 64);
}

const SIZE: usize = 1024;

#[test]
fn writes_and_reads_data() {
    let buffer = RingBuffer::<i64, SIZE>::new();

    for i in 0..SIZE as i64 {
        unsafe {
            *buffer.get_mut(i as Sequence) = i;
        }
    }

    for i in 0..SIZE as i64 {
        unsafe {
            assert_eq!(*buffer.get(i as Sequence), i);
        }
    }
}

#[test]
fn writes_are_visible_across_threads() {
    let buffer: Arc<RingBuffer<i64, SIZE>> = Arc::new(RingBuffer::new());

    let b1 = buffer.clone();
    let t1 = std::thread::spawn(move || {
        for i in 0..SIZE as i64 {
            unsafe {
                *b1.get_mut(i as Sequence) = i;
            }
        }
    });

    let b2 = buffer.clone();
    let t2 = std::thread::spawn(move || {
        for i in 0..SIZE as i64 {
            unsafe {
                *b2.get_mut(i as Sequence) = i * 2;
            }
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();
}
