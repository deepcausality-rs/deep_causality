// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::WindowStorage;

#[derive(Debug)]
pub struct UnsafeVectorStorage<T>
where
    T: PartialEq + Copy,
{
    vec: Vec<T>,
    size: usize,
    head: usize,
    tail: usize,
}

impl<T> UnsafeVectorStorage<T>
where
    T: PartialEq + Copy + Default,
{
    pub fn new(size: usize, multiple: usize) -> Self {
        Self {
            vec: Vec::with_capacity(size * multiple),
            size,
            head: 0,
            tail: 0,
        }
    }
}

impl<T> WindowStorage<T> for UnsafeVectorStorage<T>
where
    T: PartialEq + Copy + Default,
{
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

    #[inline(always)]
    fn last(&self) -> Result<T, String> {
        if self.filled() {
            Ok(self.vec[self.tail - 1])
        } else {
            Err("Vector is not yet filled. Add some elements to the array first".to_string())
        }
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
            // Adjust offset
            &self.vec[self.head + 1..self.tail]
        } else {
            &self.vec[self.head..self.tail]
        }
    }
}
