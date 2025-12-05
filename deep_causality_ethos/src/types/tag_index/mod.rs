/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{TeloidID, TeloidTag};
use std::collections::{HashMap, HashSet};

mod display;
mod index;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TagIndex {
    index: HashMap<TeloidTag, HashSet<TeloidID>>,
}

impl TagIndex {
    /// Creates a new, empty `TagIndex`.
    ///
    /// # Returns
    ///
    /// A new `TagIndex` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::TagIndex;
    /// let tag_index = TagIndex::new();
    /// ```
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Creates a new `TagIndex` with a specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The initial capacity of the index.
    ///
    /// # Returns
    ///
    /// A new `TagIndex` instance with the given capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::TagIndex;
    /// let tag_index = TagIndex::with_capacity(10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
        }
    }
}
