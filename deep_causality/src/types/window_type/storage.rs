/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

pub trait WindowStorage<T>
    where
        T: PartialEq + Copy + Default
{
    /// Pushes a new element to the beginning of the sliding window.
    /// If the window is filled, the last element will be dropped.
    fn push(&mut self, value: T);
    /// Returns the first element in the sliding window
    fn first(&self) -> Result<T, String>;
    /// Returns the last element in the sliding window
    fn last(&self) -> Result<T, String>;
    /// Returns tail cursor
    fn tail(&self) -> usize;
    /// Returns size
    fn size(&self) -> usize;
    /// Returns sliding window as slice
    fn get_slice(&self) -> &[T];

    //
    // Default implementations. Override as required.
    //

    /// Returns true if the window is empty.
    fn empty(&self) -> bool {
        return if self.tail() == 0 {
            true
        } else {
            false
        };
    }

    /// Returns true if the window is filled.
    fn filled(&self) -> bool
    {
        return if self.tail() < self.size() {
            false
        } else {
            true
        };
    }

    /// Returns the sliding window as a fixed size static array.
    fn arr<const S: usize>(&self) -> Result<[T; S], String> {
        if !self.filled() {
            return Err(format!("Sliding window is not yet filled. Add some elements to the array first"));
        }

        let mut arr: [T; S] = [T::default(); S];
        let slice = self.get_slice();
        for i in 0..self.size() {
            arr[i] = slice[i];
        }

        Ok(arr)
    }

    /// Returns the sliding window as a slice.
    fn slice(&self) -> Result<&[T], String> {
        return if !self.filled() {
            Err(format!("Sliding window is not yet filled. Add some elements to the array first"))
        } else {
            Ok(self.get_slice())
        };
    }

    /// Returns the sliding window as a vector.
    fn vec(&self) -> Result<Vec<T>, String> {
        return if !self.filled() {
            Err(format!("Sliding window is not yet filled. Add some elements to the array first"))
        } else {
            Ok(self.get_slice().to_vec())
        };
    }
}