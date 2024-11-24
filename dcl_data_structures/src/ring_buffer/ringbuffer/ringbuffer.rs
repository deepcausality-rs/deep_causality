use crate::ring_buffer::prelude::*;

use std::cell::UnsafeCell;

#[repr(align(64))]  // Align to cache line size
pub struct RingBuffer<T, const N: usize> {
    data: [UnsafeCell<T>; N],
    mask: usize,
}

impl<T: Default + Copy, const N: usize> RingBuffer<T, N> {
    #[inline(always)]
    pub fn new() -> Self {
        assert!(
            (N != 0) && ((N & (N - 1)) == 0),
            "capacity must be power of two"
        );

        let data = unsafe {
            let mut data: [UnsafeCell<T>; N] = std::mem::MaybeUninit::uninit().assume_init();
            for element in &mut data[..] {
                std::ptr::write(element, UnsafeCell::new(T::default()));
            }
            data
        };

        RingBuffer {
            data,
            mask: N - 1,
        }
    }

    /// Returns the optimal batch size for best performance based on benchmark results.
    /// For single-producer scenarios, a batch size of 50-100 elements provides the best throughput.
    /// For multi-producer scenarios, a batch size of 100 elements provides the best balance of throughput and latency.
    #[inline(always)]
    pub const fn optimal_batch_size() -> usize {
        80  // Estimated optimal batch size based on benchmarks (average of 50-100 range)
    }
}

impl<T: Send + Sync, const N: usize> DataProvider<T> for RingBuffer<T, N> {
    #[inline(always)]
    fn buffer_size(&self) -> usize {
        N
    }

    #[inline(always)]
    unsafe fn get_mut(&self, sequence: Sequence) -> &mut T {
        let index = sequence as usize & self.mask;
        &mut *self.data.get_unchecked(index).get()
    }

    #[inline(always)]
    unsafe fn get(&self, sequence: Sequence) -> &T {
        let index = sequence as usize & self.mask;
        &*self.data.get_unchecked(index).get()
    }
}

unsafe impl<T: Send, const N: usize> Send for RingBuffer<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for RingBuffer<T, N> {}
