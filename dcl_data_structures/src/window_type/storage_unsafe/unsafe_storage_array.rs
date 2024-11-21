// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::WindowStorage;
use std::mem::align_of;

const ERR_EMPTY: &str = "Array is empty. Add some elements to the array first";
const ERR_NOT_FILLED: &str = "Array is not yet filled. Add some elements to the array first";

/// High-performance array-based storage using unsafe operations
///
/// # Implementation Notes
/// - Uses 64-byte alignment for cache optimization
/// - Maintains cached array pointer for fast access
/// - Implements optimized memory operations for 4+ byte types
#[cfg(feature = "unsafe")]
#[repr(C, align(64))]
#[derive(Debug)]
pub struct UnsafeArrayStorage<T, const SIZE: usize, const CAPACITY: usize>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
    arr: [T; CAPACITY],
    ptr: *mut T,
    size: usize,
    head: usize,
    tail: usize,
}

#[cfg(feature = "unsafe")]
impl<T, const SIZE: usize, const CAPACITY: usize> UnsafeArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
    /// Creates a new UnsafeArrayStorage instance
    ///
    /// # Implementation Notes
    /// - Initializes array with default values
    /// - Caches array pointer for optimized access
    /// - Requires 4-byte alignment for optimal performance
    #[inline(always)]
    pub fn new() -> Self {
        assert!(CAPACITY > SIZE, "CAPACITY must be greater than SIZE");
        assert!(align_of::<T>() >= 4, "Type must be at least 4-byte aligned");
        let mut storage = Self {
            arr: [T::default(); CAPACITY],
            ptr: std::ptr::null_mut(),
            size: SIZE,
            head: 0,
            tail: 0,
        };
        storage.ptr = storage.arr.as_mut_ptr();
        storage
    }

    #[inline(always)]
    const fn is_full(&self) -> bool {
        self.tail >= CAPACITY
    }

    #[inline(always)]
    const fn needs_head_adjustment(&self) -> bool {
        self.tail.saturating_sub(self.head) > self.size
    }

    /// Rewinds storage by copying elements to array start
    ///
    /// # Implementation Notes
    /// - Uses optimized copying for 4+ byte types
    /// - Copies in 16-byte chunks when possible
    /// - Falls back to standard copy for smaller types
    #[inline(always)]
    unsafe fn rewind(&mut self) {
        // Use optimized copy for larger types
        if std::mem::size_of::<T>() >= 4 && align_of::<T>() >= 4 {
            let src = self.ptr.add(self.head);
            let dst = self.ptr;

            // Copy in chunks of 16 bytes when possible
            let simd_chunks = (self.size - 1) / 4;
            if simd_chunks > 0 {
                std::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, simd_chunks * 16);

                // Copy remaining elements
                let remaining = (self.size - 1) % 4;
                if remaining > 0 {
                    std::ptr::copy_nonoverlapping(
                        src.add(simd_chunks * 4),
                        dst.add(simd_chunks * 4),
                        remaining,
                    );
                }
            } else {
                std::ptr::copy_nonoverlapping(src, dst, self.size - 1);
            }
        } else {
            // Fallback for smaller types
            std::ptr::copy_nonoverlapping(self.ptr.add(self.head), self.ptr, self.size - 1);
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
    /// Creates a default UnsafeArrayStorage instance
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
    /// Pushes a new value into storage
    ///
    /// # Args
    /// * `value` - Value to push
    ///
    /// # Implementation Notes
    /// - Uses direct pointer access for performance
    /// - Automatically rewinds when full
    /// - Adjusts head when window size exceeded
    #[inline(always)]
    fn push(&mut self, value: T) {
        unsafe {
            if self.is_full() {
                self.rewind();
            }

            *self.ptr.add(self.tail) = value;

            if self.needs_head_adjustment() {
                self.head += 1;
            }

            self.tail += 1;
        }
    }

    /// Returns first element in window
    ///
    /// # Errors
    /// Returns error if storage is empty
    ///
    /// # Implementation Notes
    /// - Uses cached pointer for fast access
    /// - Handles both normal and wrapped states
    #[inline(always)]
    fn first(&self) -> Result<T, String> {
        if self.tail == 0 {
            return Err(ERR_EMPTY.to_string());
        }

        unsafe {
            Ok(if self.tail > self.size {
                *self.ptr.add(self.head + 1)
            } else {
                *self.ptr.add(self.head)
            })
        }
    }

    /// Returns last element in window
    ///
    /// # Errors
    /// Returns error if storage not filled
    ///
    /// # Implementation Notes
    /// - Uses cached pointer for fast access
    /// - Verifies fill state before access
    #[inline(always)]
    fn last(&self) -> Result<T, String> {
        if !self.filled() {
            return Err(ERR_NOT_FILLED.to_string());
        }

        unsafe { Ok(*self.ptr.add(self.tail - 1)) }
    }

    /// Returns current tail position
    #[inline(always)]
    fn tail(&self) -> usize {
        self.tail
    }

    /// Returns window size
    #[inline(always)]
    fn size(&self) -> usize {
        self.size
    }

    /// Returns slice of current window contents
    ///
    /// # Implementation Notes
    /// - Creates slice from raw pointers for performance
    /// - Handles both normal and wrapped states
    /// - Uses cached pointer to avoid conversions
    #[inline(always)]
    fn get_slice(&self) -> &[T] {
        unsafe {
            if self.tail > self.size {
                std::slice::from_raw_parts(self.ptr.add(self.head + 1), self.tail - (self.head + 1))
            } else {
                std::slice::from_raw_parts(self.ptr.add(self.head), self.tail - self.head)
            }
        }
    }
}
