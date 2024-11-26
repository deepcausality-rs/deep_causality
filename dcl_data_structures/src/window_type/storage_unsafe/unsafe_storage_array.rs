// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

#[cfg(feature = "unsafe")]
use crate::prelude::WindowStorage;

#[cfg(feature = "unsafe")]
const ERROR_EMPTY_ARRAY: &str = "Array is empty";

#[cfg(feature = "unsafe")]
const ERROR_ARRAY_NOT_FILLED: &str = "Array is not yet filled";

/// A high-performance, unsafe implementation of a fixed-size sliding window storage using a raw array.
/// This implementation prioritizes performance by using unsafe Rust features and manual memory management.
///
/// # Type Parameters
/// - `T`: The type of elements stored in the array
/// - `SIZE`: The logical size of the sliding window
/// - `CAPACITY`: The physical capacity of the underlying array
///
/// # Constraints
/// - `T` must implement `PartialEq + Copy + Default`
/// - `CAPACITY` must be greater than `SIZE`
/// - The array `[T; CAPACITY]` must be sized
///
/// # Memory Layout
/// The structure is aligned to 64 bytes for optimal cache line usage.
///
#[cfg(feature = "unsafe")]
#[repr(C, align(64))]
#[derive(Debug)]
pub struct UnsafeArrayStorage<T, const SIZE: usize, const CAPACITY: usize>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
    /// The underlying array storing the elements
    arr: [T; CAPACITY],
    /// Index of the first element in the window
    head: usize,
    /// Index where the next element will be inserted
    tail: usize,
    /// The logical size of the sliding window
    size: usize,
}

#[cfg(feature = "unsafe")]
impl<T, const SIZE: usize, const CAPACITY: usize> UnsafeArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
    /// Creates a new `UnsafeArrayStorage` instance.
    ///
    /// # Panics
    /// Panics if `CAPACITY` is not greater than `SIZE`.
    ///
    #[inline(always)]
    pub fn new() -> Self {
        assert!(CAPACITY > SIZE, "CAPACITY must be greater than SIZE");
        Self {
            arr: [T::default(); CAPACITY],
            head: 0,
            tail: 0,
            size: SIZE,
        }
    }

    /// Checks if the storage contains the full window size of elements.
    ///
    /// # Safety
    /// Uses unchecked subtraction for performance.
    #[inline(always)]
    fn filled(&self) -> bool {
        unsafe { self.tail.unchecked_sub(self.head) >= self.size }
    }

    /// Rewinds the storage by copying the last `SIZE` elements to the beginning.
    ///
    /// This method uses optimized copying strategies:
    /// - For types >= 4 bytes: Uses 16-byte chunk copying
    /// - For smaller types: Falls back to standard copying
    ///
    /// # Safety
    /// Uses unsafe pointer manipulation and unchecked arithmetic.
    ///
    #[inline(always)]
    fn rewind(&mut self) {
        unsafe {
            let type_size = std::mem::size_of::<T>();
            let src = self.arr.as_ptr().add(self.tail - self.size);
            let dst = self.arr.as_mut_ptr();

            if type_size >= 4 {
                // For 4+ byte types, use optimized copying
                let bytes_to_copy = self.size * type_size;
                let chunks_16 = bytes_to_copy / 16;
                let remainder = bytes_to_copy % 16;

                // Copy 16-byte chunks
                if chunks_16 > 0 {
                    let src_bytes = src as *const u8;
                    let dst_bytes = dst as *mut u8;
                    for i in 0..chunks_16 {
                        std::ptr::copy_nonoverlapping(
                            src_bytes.add(i * 16),
                            dst_bytes.add(i * 16),
                            16,
                        );
                    }
                }

                // Copy remaining bytes
                if remainder > 0 {
                    let src_bytes = (src as *const u8).add(chunks_16 * 16);
                    let dst_bytes = (dst as *mut u8).add(chunks_16 * 16);
                    std::ptr::copy_nonoverlapping(src_bytes, dst_bytes, remainder);
                }
            } else {
                // Fall back to standard copy for smaller types
                std::ptr::copy_nonoverlapping(src, dst, self.size);
            }
        }
        self.head = 0;
        self.tail = self.size;
    }
}

#[cfg(feature = "unsafe")]
impl<T, const SIZE: usize, const CAPACITY: usize> Default for UnsafeArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; SIZE]: Sized,
{
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "unsafe")]
impl<T, const SIZE: usize, const CAPACITY: usize> WindowStorage<T>
    for UnsafeArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; SIZE]: Sized,
{
    /// Pushes a new value into the storage.
    /// Drops old values if the storage is full relative to its size.
    ///
    /// If the storage is full, it automatically rewinds by moving the last `SIZE`
    /// elements to the beginning of the array.
    ///
    /// # Safety
    /// Uses unchecked operations for performance.
    #[inline(always)]
    fn push(&mut self, value: T) {
        unsafe {
            if self.tail >= CAPACITY {
                self.rewind();
            }

            *self.arr.get_unchecked_mut(self.tail) = value;
            self.tail = self.tail.wrapping_add(1);

            if self.tail.unchecked_sub(self.head) > self.size {
                self.head = self.tail.unchecked_sub(self.size);
            }
        }
    }

    /// Returns the first element in the storage.
    ///
    /// # Errors
    /// Returns an error if the storage is empty.
    ///
    /// # Safety
    /// Uses unchecked array access for performance.
    #[inline(always)]
    fn first(&self) -> Result<T, String> {
        if self.tail == 0 {
            return Err(ERROR_EMPTY_ARRAY.to_string());
        }
        unsafe { Ok(*self.arr.get_unchecked(self.head)) }
    }

    /// Returns the last element in the storage.
    ///
    /// # Errors
    /// Returns an error if the storage is not filled to size.
    ///
    /// # Safety
    /// Uses unchecked array access for performance.
    #[inline(always)]
    fn last(&self) -> Result<T, String> {
        if !self.filled() {
            return Err(ERROR_ARRAY_NOT_FILLED.to_string());
        }
        unsafe { Ok(*self.arr.get_unchecked(self.tail - 1)) }
    }

    /// Returns the current tail index.
    #[inline(always)]
    fn tail(&self) -> usize {
        self.tail
    }

    /// Returns the size of the sliding window.
    #[inline(always)]
    fn size(&self) -> usize {
        self.size
    }

    /// Returns a slice of the current window contents.
    ///
    /// # Safety
    /// Uses unsafe raw pointer manipulation for creating the slice.
    #[inline(always)]
    fn get_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.arr.as_ptr().add(self.head),
                self.tail.saturating_sub(self.head).min(self.size),
            )
        }
    }
}
