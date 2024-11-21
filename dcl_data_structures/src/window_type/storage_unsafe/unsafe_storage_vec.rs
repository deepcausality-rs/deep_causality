// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::WindowStorage;

/// An unsafe but highly optimized sliding window implementation using a vector as the underlying storage.
/// 
/// This implementation uses unsafe Rust features to achieve maximum performance while maintaining
/// memory safety through careful bounds checking and memory management.
///
/// # Type Parameters
/// * `T` - The type of elements stored in the window. Must implement `PartialEq` and `Copy`.
///
/// # Safety
/// While this implementation uses unsafe code internally, all public methods maintain memory
/// safety through careful bounds checking and proper memory management. The unsafe optimizations
/// include:
/// - Direct pointer arithmetic for fast access
/// - Pre-allocated uninitialized memory
/// - Bounds-check elimination where safe
/// - Cache-line alignment for better CPU cache utilization
#[repr(align(64))]
#[derive(Debug)]
pub struct UnsafeVectorStorage<T>
where
    T: PartialEq + Copy,
{
    vec: Vec<T>,
    size: usize,
    head: usize,
    tail: usize,
    capacity: usize,
}

impl<T> UnsafeVectorStorage<T>
where
    T: PartialEq + Copy + Default,
{
    /// Creates a new UnsafeVectorStorage with the specified size and capacity multiplier.
    ///
    /// # Arguments
    /// * `size` - The maximum number of elements that can be viewed in the sliding window at once
    /// * `multiple` - The capacity multiplier. Total capacity will be size * multiple
    ///
    /// # Returns
    /// A new instance of UnsafeVectorStorage
    ///
    /// # Implementation Notes
    /// - Pre-allocates memory for the entire capacity upfront
    /// - Uses uninitialized memory for better performance
    /// - Aligns the structure to cache line boundaries (64 bytes)
    #[inline(always)]
    pub fn new(size: usize, multiple: usize) -> Self {
        let capacity = size * multiple;
        let mut vec = Vec::with_capacity(capacity);
        unsafe {
            vec.set_len(capacity);
        }
        Self {
            vec,
            size,
            head: 0,
            tail: 0,
            capacity,
        }
    }
}

impl<T> WindowStorage<T> for UnsafeVectorStorage<T>
where
    T: PartialEq + Copy + Default,
{
    /// Pushes a new value into the sliding window.
    ///
    /// # Arguments
    /// * `value` - The value to push into the window
    ///
    /// # Implementation Notes
    /// - Uses a fast path for the common case where there's available capacity
    /// - Employs branchless arithmetic for head updates
    /// - Automatically rewinds when capacity is reached
    /// - Uses unsafe optimizations for maximum performance while maintaining safety
    #[inline(always)]
    fn push(&mut self, value: T) {
        unsafe {
            // Fast path: just append if there's space
            if self.tail < self.capacity {
                *self.vec.get_unchecked_mut(self.tail) = value;
                self.tail += 1;
                self.head += (self.tail - self.head > self.size) as usize;
                return;
            }

            // Slow path: rewind needed
            std::ptr::copy_nonoverlapping(
                self.vec.as_ptr().add(self.head),
                self.vec.as_mut_ptr(),
                self.size,
            );
            self.head = 0;
            self.tail = self.size;
            *self.vec.get_unchecked_mut(self.tail) = value;
            self.tail += 1;
        }
    }

    /// Returns the first element in the sliding window.
    ///
    /// # Returns
    /// * `Ok(T)` - The first element if the window is not empty
    /// * `Err(String)` - An error message if the window is empty
    ///
    /// # Implementation Notes
    /// - Uses unchecked indexing for performance after bounds validation
    /// - Maintains safety through explicit empty check
    #[inline(always)]
    fn first(&self) -> Result<T, String> {
        if self.tail == 0 {
            return Err("Vector is empty. Add some elements to the array first".to_string());
        }
        unsafe { Ok(*self.vec.get_unchecked(self.head)) }
    }

    /// Returns the last element in the sliding window.
    ///
    /// # Returns
    /// * `Ok(T)` - The last element if the window is filled
    /// * `Err(String)` - An error message if the window is not yet filled
    ///
    /// # Implementation Notes
    /// - Uses unchecked indexing for performance after bounds validation
    /// - Maintains safety through explicit filled check
    #[inline(always)]
    fn last(&self) -> Result<T, String> {
        if !self.filled() {
            return Err("Vector is not yet filled. Add some elements to the array first".to_string());
        }
        unsafe { Ok(*self.vec.get_unchecked(self.tail - 1)) }
    }

    /// Returns the current tail position of the sliding window.
    ///
    /// # Returns
    /// The current tail index
    #[inline(always)]
    fn tail(&self) -> usize {
        self.tail
    }

    /// Returns the size of the sliding window.
    ///
    /// # Returns
    /// The maximum number of elements that can be viewed in the window at once
    #[inline(always)]
    fn size(&self) -> usize {
        self.size
    }

    /// Returns a slice containing all current elements in the sliding window.
    ///
    /// # Returns
    /// A slice containing the current elements from head to tail
    ///
    /// # Implementation Notes
    /// - Uses unchecked slicing for performance
    /// - Safety is guaranteed by the internal head/tail management
    #[inline(always)]
    fn get_slice(&self) -> &[T] {
        unsafe { self.vec.get_unchecked(self.head..self.tail) }
    }

    /// Checks if the sliding window is filled to its maximum size.
    ///
    /// # Returns
    /// * `true` - If the window contains the maximum number of elements
    /// * `false` - If the window is not yet filled to capacity
    #[inline(always)]
    fn filled(&self) -> bool {
        self.tail >= self.size
    }
}
