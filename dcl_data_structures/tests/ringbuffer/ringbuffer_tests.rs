use dcl_data_structures::ring_buffer::prelude::*;
use std::sync::Arc;

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
