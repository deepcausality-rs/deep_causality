// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::marker::PhantomData;
use crate::prelude::{ArrayStorage, VectorStorage, WindowStorage};

pub fn new_with_vector_storage<T: PartialEq + Copy + Default>(
    size: usize,
    multiple: usize,
)
    -> SlidingWindow<VectorStorage<T>, T>
{
    SlidingWindow::with_storage(
        VectorStorage::new(size, multiple)
    )
}

pub fn new_with_array_storage<T: PartialEq + Copy + Default, const SIZE: usize, const CAPACITY: usize>()
    -> SlidingWindow<ArrayStorage<T, SIZE, CAPACITY>, T>
{
    assert!(CAPACITY > SIZE);

    SlidingWindow::with_storage(
        ArrayStorage::new()
    )
}

pub struct SlidingWindow<S, T>
    where
        T: PartialEq + Copy + Default,
        S: WindowStorage<T>,
{
    storage: S,
    ty: PhantomData<T>,
}

impl<S, T> SlidingWindow<S, T>
    where
        T: PartialEq + Copy + Default,
        S: WindowStorage<T>,
{
    pub(crate) fn with_storage(storage: S) -> Self
    {
        Self { storage, ty: Default::default() }
    }
}


impl<S, T> SlidingWindow<S, T>
    where
        T: PartialEq + Copy + Default,
        S: WindowStorage<T>,
{
    /// Pushes a new element to the beginning of the sliding window.
    /// If the window is filled, the last element will be dropped.
    pub fn push(&mut self, value: T)
    {
        self.storage.push(value)
    }
    /// Returns the first element in the sliding window
    pub fn first(&self) -> Result<T, String>
    {
        self.storage.first()
    }
    /// Returns the last element in the sliding window
    pub fn last(&self) -> Result<T, String>
    {
        self.storage.last()
    }
    /// Returns true if the window is empty.
    pub fn empty(&self) -> bool
    {
        self.storage.empty()
    }
    /// Returns true if the window is filled.
    pub fn filled(&self) -> bool
    {
        self.storage.filled()
    }
    /// Returns the window size
    pub fn size(&self) -> usize
    {
        self.storage.size()
    }
    /// Returns the sliding window as a fixed size static array.
    pub fn arr<const SIZE: usize>(&self) -> Result<[T; SIZE], String> {
        self.storage.arr()
    }
    /// Returns sliding window as slice
    pub fn slice(&self) -> Result<&[T], String> {
        self.storage.slice()
    }
    /// Returns the sliding window as a vector.
    pub fn vec(&self) -> Result<Vec<T>, String> {
        self.storage.vec()
    }
}