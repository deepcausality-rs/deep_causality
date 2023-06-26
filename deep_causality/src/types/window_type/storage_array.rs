// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use deep_causality_macros::{make_first, make_get_slice, make_last, make_size, make_tail};
use crate::prelude::WindowStorage;

pub struct ArrayStorage<T, const SIZE: usize, const CAPACITY: usize>
    where
        T: PartialEq + Copy + Default,
        [T; CAPACITY]: Sized,
{
    store: [T; CAPACITY],
    size: usize,
    head: usize,
    tail: usize,
}

impl<T, const SIZE: usize, const CAPACITY: usize> ArrayStorage<T, SIZE, CAPACITY>
    where
        T: PartialEq + Copy + Default,
        [T; CAPACITY]: Sized,
{
    pub fn new() -> Self
    {
        Self { store: [T::default(); CAPACITY], size: SIZE, head: 0, tail: 0, }
    }
}

impl<T, const SIZE: usize, const CAPACITY: usize> WindowStorage<T> for ArrayStorage<T, SIZE, CAPACITY>
    where
        T: PartialEq + Copy + Default,
        [T; SIZE]: Sized,
{
    fn push(&mut self, value: T) {
        // if the array is full, rewind
        if self.tail > 0 && self.tail == self.store.len()
        {
            // rewind
            for i in 0..self.size - 1
            {
                self.store[i] = self.store[self.head + i];
            }
            self.head = 0;
            self.tail = self.size;
        }

        // push the value
        self.store[self.tail] = value;

        // check if the window is full,
        if self.tail - self.head > self.size
        {
            // move head cursor one position forward
            self.head += 1;
        }

        //increase tail cursor to next position
        self.tail += 1;
    }

    // macro generated implementations of trait methods.
    make_first!();
    make_last!();
    make_tail!();
    make_size!();
    make_get_slice!();
}