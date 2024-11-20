// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::WindowStorage;

/// A vector-based sliding window implementation
///
/// # Implementation Note
/// Uses a Vec<T> as the underlying storage mechanism with dynamic resizing capabilities.
/// The window maintains a head and tail pointer to track the active window region.
#[derive(Debug)]
pub struct VectorStorage<T>
where
    T: PartialEq + Copy,
{
    vec: Vec<T>,
    size: usize,
    head: usize,
    tail: usize,
}

impl<T> VectorStorage<T>
where
    T: PartialEq + Copy + Default,
{
    /// Creates a new VectorStorage instance with the specified size and multiple.
    ///
    /// # Args
    /// * `size` - The size of the sliding window
    /// * `multiple` - The multiple of the size for the underlying vector capacity
    ///
    /// # Returns
    /// * `Self` - A new VectorStorage instance
    pub fn new(size: usize, multiple: usize) -> Self {
        Self {
            vec: Vec::with_capacity(size * multiple),
            size,
            head: 0,
            tail: 0,
        }
    }
}

impl<T> WindowStorage<T> for VectorStorage<T>
where
    T: PartialEq + Copy + Default,
{
    /// Pushes a new element into the sliding window
    ///
    /// # Args
    /// * `value` - The value to be pushed into the window
    ///
    /// # Implementation Note
    /// If the window is full (tail == capacity), it performs a rewind operation using copy_within
    /// to maintain the sliding window invariant.
    #[inline(always)]
    fn push(&mut self, value: T) {
        // if the array is full, rewind
        if self.tail > 0 && self.tail == self.vec.capacity() {
            // rewind
            self.vec
                .copy_within(self.head..self.head + self.size - 1, 0);
            self.head = 0;
            self.tail = self.size;
        }

        self.vec.push(value); // store the value

        // check if the window is full,
        if self.tail - self.head > self.size {
            // if so, move head cursor one position forward
            self.head += 1;
        }

        // otherwise increase tail cursor to next position
        self.tail += 1;
    }

    /// Returns the first (oldest) element in the sliding window
    ///
    /// # Returns
    /// * `Ok(T)` - The first element in the window
    /// * `Err(String)` - If the window is empty
    ///
    /// # Implementation Note
    /// Takes into account the window's head position and size to return the correct element
    #[inline(always)]
    fn first(&self) -> Result<T, String> {
        if self.tail != 0 {
            if self.tail > self.size {
                Ok(self.vec[self.head + 1])
            } else {
                Ok(self.vec[self.head])
            }
        } else {
            Err("Vector is empty. Add some elements to the array first".to_string())
        }
    }

    /// Returns the last (newest) element in the sliding window
    ///
    /// # Returns
    /// * `Ok(T)` - The last element in the window
    /// * `Err(String)` - If the window is not yet filled
    ///
    /// # Implementation Note
    /// Uses the tail position to determine the newest element
    #[inline(always)]
    fn last(&self) -> Result<T, String> {
        if self.filled() {
            Ok(self.vec[self.tail - 1])
        } else {
            Err("Vector is not yet filled. Add some elements to the array first".to_string())
        }
    }

    /// Returns the current tail position of the window
    ///
    /// # Returns
    /// * `usize` - The current tail position
    ///
    /// # Implementation Note
    /// The tail position indicates where the next element will be inserted
    #[inline(always)]
    fn tail(&self) -> usize {
        self.tail
    }

    /// Returns the size of the sliding window
    ///
    /// # Returns
    /// * `usize` - The configured size of the sliding window
    ///
    /// # Implementation Note
    /// This is the fixed size specified during construction
    #[inline(always)]
    fn size(&self) -> usize {
        self.size
    }

    /// Returns a slice of the current window contents
    ///
    /// # Returns
    /// * `&[T]` - A slice containing the current window elements
    ///
    /// # Implementation Note
    /// Returns a slice from the underlying vector based on head position and size
    #[inline(always)]
    fn get_slice(&self) -> &[T] {
        if self.tail > self.size {
            // Adjust offset
            &self.vec[self.head + 1..self.tail]
        } else {
            &self.vec[self.head..self.tail]
        }
    }
}
