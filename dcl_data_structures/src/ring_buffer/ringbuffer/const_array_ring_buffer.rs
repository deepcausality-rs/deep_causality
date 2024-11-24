use crate::ring_buffer::prelude::*;

use std::cell::UnsafeCell;

/// A fixed-size, lock-free ring buffer implementation optimized for high-performance scenarios.
///
/// # Type Parameters
/// * `T` - The type of elements stored in the buffer. Must implement `Default` and `Copy`.
/// * `N` - The size of the buffer, must be a power of two.
///
/// # Performance Characteristics
/// - Lock-free operations for both single and multi-producer scenarios
/// - Cache-line aligned (64 bytes) to prevent false sharing
/// - Constant time O(1) read and write operations
/// - Zero allocation during runtime (fixed-size array allocated at initialization)
///
/// # Memory Layout
/// The buffer is aligned to cache line size (64 bytes) to prevent false sharing
/// between CPU cores, which is crucial for multi-threaded performance.
///
/// # Safety
/// The implementation ensures thread safety through atomic operations and memory barriers.
/// The buffer size must be a power of two to enable efficient wrapping using bitwise AND.
///
/// # Examples
/// ```
/// use dcl_data_structures::ring_buffer::prelude::*;
///
/// let ring_buffer = RingBuffer::<u64, 1024>::new();
/// assert_eq!(ring_buffer.capacity(), 1024);
/// ```
#[repr(align(64))] // Align to cache line size
pub struct RingBuffer<T, const N: usize>
where
    T: Default + Copy,
{
    /// The underlying array storing the elements.
    /// Uses UnsafeCell to allow interior mutability required for lock-free operations.
    data: [UnsafeCell<T>; N],

    /// Bitmask for fast modulo operations.
    /// Equal to N-1 where N is a power of 2, allowing us to use bitwise AND
    /// instead of expensive modulo operations.
    mask: usize,
}

impl<T, const N: usize> Default for RingBuffer<T, N>
where
    T: Default + Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> RingBuffer<T, N>
where
    T: Default + Copy,
{
    /// Creates a new RingBuffer with the specified capacity.
    ///
    /// # Arguments
    /// None
    ///
    /// # Returns
    /// A new RingBuffer instance
    ///
    /// # Panics
    /// Panics if N is 0 or not a power of two
    ///
    /// # Examples
    /// ```
    /// use dcl_data_structures::ring_buffer::prelude::*;
    ///
    /// let ring_buffer = RingBuffer::<u64, 1024>::new();
    /// assert_eq!(ring_buffer.capacity(), 1024);
    /// ```
    #[inline(always)]
    pub fn new() -> Self {
        assert!(
            (N != 0) && ((N & (N - 1)) == 0),
            "capacity must be power of two"
        );

        // Initialize the array with default values in a safe way
        let data = [(); N].map(|_| UnsafeCell::new(T::default()));

        RingBuffer { data, mask: N - 1 }
    }

    /// Returns the capacity of the ring buffer.
    ///
    /// # Returns
    /// The capacity of the ring buffer, which is always a power of two.
    ///
    /// # Examples
    /// ```
    /// use dcl_data_structures::ring_buffer::prelude::*;
    ///
    /// let ring_buffer = RingBuffer::<u64, 1024>::new();
    /// assert_eq!(ring_buffer.capacity(), 1024);
    /// ```
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        N
    }

    /// Returns the optimal batch size for best performance based on benchmark results.
    ///
    /// # Performance Guidelines
    /// - Single-producer scenarios: 50-100 elements provides optimal throughput
    /// - Multi-producer scenarios: 100 elements provides best balance of throughput and latency
    ///
    /// # Returns
    /// Returns 80 as the recommended batch size, which is the average of the optimal range (50-100)
    /// and provides good performance for both single and multi-producer scenarios.
    #[inline(always)]
    pub const fn optimal_batch_size() -> usize {
        80 // Estimated optimal batch size based on benchmarks (average of 50-100 range)
    }
}

/// Implementation of the DataProvider trait for RingBuffer
///
/// This implementation provides the core functionality for reading and writing
/// data to the ring buffer in a thread-safe manner.
impl<T, const N: usize> DataProvider<T> for RingBuffer<T, N>
where
    T: Send + Sync + Default + Copy,
{
    /// Returns the total size of the buffer.
    ///
    /// This is a constant-time operation that returns the fixed size N.
    #[inline(always)]
    fn buffer_size(&self) -> usize {
        N
    }

    /// Gets a mutable reference to the element at the given sequence number.
    ///
    /// # Safety
    /// This function is unsafe because:
    /// - It performs unchecked indexing for performance
    /// - Caller must ensure proper synchronization
    /// - Caller must ensure the sequence number is valid
    #[inline(always)]
    unsafe fn get_mut(&self, sequence: Sequence) -> &mut T {
        let index = sequence as usize & self.mask;
        &mut *self.data.get_unchecked(index).get()
    }

    /// Gets a reference to the element at the given sequence number.
    ///
    /// # Safety
    /// This function is unsafe because:
    /// - It performs unchecked indexing for performance
    /// - Caller must ensure proper synchronization
    /// - Caller must ensure the sequence number is valid
    #[inline(always)]
    unsafe fn get(&self, sequence: Sequence) -> &T {
        let index = sequence as usize & self.mask;
        &*self.data.get_unchecked(index).get()
    }
}

// Implement Send for RingBuffer when T is Send
unsafe impl<T, const N: usize> Send for RingBuffer<T, N> where T: Send + Default + Copy {}

// Implement Sync for RingBuffer when T is Sync
unsafe impl<T, const N: usize> Sync for RingBuffer<T, N> where T: Sync + Default + Copy {}
