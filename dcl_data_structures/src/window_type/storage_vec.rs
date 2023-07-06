// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use deep_causality_macros::{make_first, make_get_slice, make_last, make_size, make_tail};
use crate::prelude::WindowStorage;

pub struct VectorStorage<T> where T: PartialEq + Copy
{
    store: Vec<T>,
    size: usize,
    head: usize,
    tail: usize,
}

impl<T> VectorStorage<T> where T: PartialEq + Copy + Default
{
    pub fn new(size: usize, multiple: usize) -> Self
    {
        Self { store: Vec::with_capacity(size * multiple), size, head: 0, tail: 0 }
    }
}

impl<T> WindowStorage<T> for VectorStorage<T> where T: PartialEq + Copy + Default
{
    fn push(&mut self, value: T) {
        // if the array is full, rewind
        if self.tail > 0 && self.tail == self.store.capacity()
        {
            // rewind
            for i in 0..self.size - 1
            {
                self.store[i] = self.store[self.head + i];
            }
            self.head = 0;
            self.tail = self.size;
        }

        self.store.push(value); // store the value

        // check if the window is full,
        if self.tail - self.head > self.size
        {
            // if so, move head cursor one position forward
            self.head += 1;
        }

        // otherwise increase tail cursor to next position
        self.tail += 1;
    }

    // All boilerplate methods generated via compiler macros.
    // See macro crate: https://crates.io/crates/deep_causality_macros
    make_first!(); make_last!(); make_tail!(); make_size!(); make_get_slice!();
}
