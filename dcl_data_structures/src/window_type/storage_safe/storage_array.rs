// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::WindowStorage;

/// A fixed-size array-based sliding window implementation
/// 
/// # Type Parameters
/// * `T` - The type of elements stored in the window
/// * `SIZE` - The size of the sliding window
/// * `CAPACITY` - The total capacity of the underlying array
/// 
/// # Implementation Note
/// Uses a fixed-size array with constant memory footprint. The window slides through
/// the array using head and tail pointers, with rewind operations when needed.
#[derive(Debug)]
pub struct ArrayStorage<T, const SIZE: usize, const CAPACITY: usize>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
    arr: [T; CAPACITY],
    size: usize,
    head: usize,
    tail: usize,
}

impl<T, const SIZE: usize, const CAPACITY: usize> ArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
    /// Creates a new ArrayStorage instance
    /// 
    /// # Returns
    /// * `Self` - A new ArrayStorage instance with initialized array and pointers
    /// 
    /// # Implementation Note
    /// Initializes a fixed-size array with default values and sets up window tracking
    #[inline(always)]
    pub fn new() -> Self {
        assert!(CAPACITY > SIZE, "CAPACITY must be greater than SIZE");
        Self {
            arr: [T::default(); CAPACITY],
            size: SIZE,
            head: 0,
            tail: 0,
        }
    }

    /// Checks if the window is full
    /// 
    /// # Returns
    /// * `bool` - True if the window is full, false otherwise
    /// 
    /// # Implementation Note
    /// Determines if the tail pointer has reached the end of the array
    #[inline(always)]
    const fn is_full(&self) -> bool {
        self.tail >= CAPACITY
    }

    /// Checks if the window needs to be rewound
    /// 
    /// # Returns
    /// * `bool` - True if the window needs rewinding, false otherwise
    /// 
    /// # Implementation Note
    /// Determines if the active window size exceeds the configured size
    #[inline(always)]
    const fn needs_head_adjustment(&self) -> bool {
        self.tail.saturating_sub(self.head) > self.size
    }

    /// Rewinds the window by copying elements to the start of the array
    /// 
    /// # Implementation Note
    /// Uses copy_within for efficient memory movement when rewinding
    #[inline(always)]
    fn rewind(&mut self) {
        self.arr
            .copy_within(self.head..self.head + self.size - 1, 0);
        self.head = 0;
        self.tail = self.size;
    }
}

impl<T, const SIZE: usize, const CAPACITY: usize> Default for ArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; SIZE]: Sized,
{
    /// Creates a new ArrayStorage instance
    /// 
    /// # Returns
    /// * `Self` - A new ArrayStorage instance with initialized array and pointers
    /// 
    /// # Implementation Note
    /// Initializes a fixed-size array with default values and sets up window tracking
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const SIZE: usize, const CAPACITY: usize> WindowStorage<T>
    for ArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; SIZE]: Sized,
{
    /// Pushes a new element into the sliding window
    /// 
    /// # Args
    /// * `value` - The value to be pushed into the window
    /// 
    /// # Implementation Note
    /// Handles window rewind when needed and maintains the sliding window invariant
    #[inline(always)]
    fn push(&mut self, value: T) {
        if self.is_full() {
            self.rewind();
        }

        self.arr[self.tail] = value;

        if self.needs_head_adjustment() {
            self.head += 1;
        }

        self.tail += 1;
    }

    /// Returns the first (oldest) element in the sliding window
    /// 
    /// # Returns
    /// * `Ok(T)` - The first element in the window
    /// * `Err(String)` - If the window is empty
    /// 
    /// # Implementation Note
    /// Accounts for window position and size to return the correct element
    #[inline(always)]
    fn first(&self) -> Result<T, String> {
        if self.tail == 0 {
            return Err("Array is empty. Add some elements to the array first".to_string());
        }

        Ok(if self.tail > self.size {
            self.arr[self.head + 1]
        } else {
            self.arr[self.head]
        })
    }

    /// Returns the last (newest) element in the sliding window
    /// 
    /// # Returns
    /// * `Ok(T)` - The last element in the window
    /// * `Err(String)` - If the window is not yet filled
    /// 
    /// # Implementation Note
    /// Uses the tail position to access the most recently added element
    #[inline(always)]
    fn last(&self) -> Result<T, String> {
        if !self.filled() {
            return Err(
                "Array is not yet filled. Add some elements to the array first".to_string(),
            );
        }

        Ok(self.arr[self.tail - 1])
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
    /// This is the fixed size specified by the SIZE generic parameter
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
    /// Returns a slice from the underlying array based on head position and size
    #[inline(always)]
    fn get_slice(&self) -> &[T] {
        if self.tail > self.size {
            &self.arr[self.head + 1..self.tail]
        } else {
            &self.arr[self.head..self.tail]
        }
    }
}
