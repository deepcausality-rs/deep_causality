// SPDX-License-Identifier: MIT
// Copyright (c) "2024" . The DeepCausality Authors. All Rights Reserved.

pub trait Indexable {
    /// Get the index for the given key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key representing the index type, typically an enum value like `TimeScale`.
    /// * `current` - Whether to get the current or previous index.
    ///
    /// # Returns
    ///
    /// The index value for the given key, typically a `usize` array position.
    ///
    fn get_index(&self, key: usize, current: bool) -> usize;

    /// Set the index for the given key.
    ///
    /// # Parameters
    ///
    /// * `key` - The key representing the index type, typically an enum value like `TimeScale`.
    /// * `index` - The index value to set, typically a `usize` array position.
    /// * `current` - Whether to set the current or previous index.
    ///
    fn set_index(&mut self, key: usize, index: usize, current: bool);
}
