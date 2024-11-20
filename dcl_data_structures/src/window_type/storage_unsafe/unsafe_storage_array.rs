// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::WindowStorage;

#[derive(Debug)]
pub struct UnsafeArrayStorage<T, const SIZE: usize, const CAPACITY: usize>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
    arr: [T; CAPACITY],
    size: usize,
    head: usize,
    tail: usize,
}

impl<T, const SIZE: usize, const CAPACITY: usize> UnsafeArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
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

    #[inline(always)]
    const fn is_full(&self) -> bool {
        self.tail >= CAPACITY
    }

    #[inline(always)]
    const fn needs_head_adjustment(&self) -> bool {
        self.tail.saturating_sub(self.head) > self.size
    }

    #[inline(always)]
    fn rewind(&mut self) {
        self.arr
            .copy_within(self.head..self.head + self.size - 1, 0);
        self.head = 0;
        self.tail = self.size;
    }
}

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

impl<T, const SIZE: usize, const CAPACITY: usize> WindowStorage<T>
    for UnsafeArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; SIZE]: Sized,
{
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

    #[inline(always)]
    fn last(&self) -> Result<T, String> {
        if !self.filled() {
            return Err(
                "Array is not yet filled. Add some elements to the array first".to_string(),
            );
        }

        Ok(self.arr[self.tail - 1])
    }

    #[inline(always)]
    fn tail(&self) -> usize {
        self.tail
    }

    #[inline(always)]
    fn size(&self) -> usize {
        self.size
    }

    #[inline(always)]
    fn get_slice(&self) -> &[T] {
        if self.tail > self.size {
            &self.arr[self.head + 1..self.tail]
        } else {
            &self.arr[self.head..self.tail]
        }
    }
}
