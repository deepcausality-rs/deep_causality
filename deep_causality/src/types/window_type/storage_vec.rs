/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */


use crate::prelude::WindowStorage;

pub struct VectorStorage<T>
    where T: PartialEq + Copy
{
    vec: Vec<T>,
    size: usize,
    head: usize,
    tail: usize,
}

impl<T> VectorStorage<T>
    where
        T: PartialEq + Copy + Default ,
{
    pub fn new(size: usize, multiple: usize) -> Self
    {
        let capacity = size * multiple;
        Self {
            vec: Vec::with_capacity(capacity),
            size,
            head: 0,
            tail: 0,
        }
    }
}

impl<T> WindowStorage<T> for VectorStorage<T>
    where
        T: PartialEq + Copy + Default ,
{
    fn push(&mut self, value: T) {
        // if the array is full, rewind
        if self.tail > 0 && self.tail == self.vec.capacity()
        {
            // rewind
            for i in 0..self.size - 1
            {
                self.vec[i] = self.vec[self.head + i];
            }
            self.head = 0;
            self.tail = self.size;
        }

        // push the value
        self.vec.push(value);

        // check if the window is full,
        if self.tail - self.head > self.size
        {
            // move head cursor one position forward
            self.head += 1;
        }

        //increase tail cursor to next position
        self.tail += 1;
    }

    fn first(&self) -> Result<T, String> {
        return if self.tail != 0 {
            Ok(self.vec[self.head])
        } else {
            Err(format!("Array is empty. Add some elements to the array first"))
        };
    }

    fn last(&self) -> Result<T, String> {
        return if self.filled() {
            Ok(self.vec[self.tail - 1])
        } else {
            Err(format!("Array is not yet filled. Add some elements to the array first"))
        };
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
    fn get_slice(&self) -> &[T]
    {
        // Adjust offset in case the window is larger than the slice.
        if self.tail > self.size
        {
            &self.vec[self.head + 1..self.tail]
        } else {
            &self.vec[self.head..self.tail]
        }
    }
}
