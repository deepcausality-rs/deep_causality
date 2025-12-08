/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{TagIndex, TeloidID, TeloidTag};
use std::collections::HashSet;

impl TagIndex {
    /// Adds a `TeloidID` to the index for a given `TeloidTag`.
    ///
    /// If the tag does not exist in the index, it will be added.
    /// If the ID already exists for the given tag, it will not be added again.
    ///
    /// # Arguments
    ///
    /// * `tag` - The `TeloidTag` to associate with the ID.
    /// * `id` - The `TeloidID` to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::{TagIndex, TeloidTag, TeloidID};
    ///
    /// let mut tag_index = TagIndex::new();
    /// let tag = "test_tag";
    /// let id = 1;
    /// tag_index.add(tag, id);
    /// tag_index.add(tag, id); // Duplicates are ignored
    /// assert_eq!(tag_index.get(tag).unwrap().len(), 1);
    /// ```
    pub fn add(&mut self, tag: TeloidTag, id: TeloidID) {
        self.index.entry(tag).or_default().insert(id);
    }

    /// Retrieves a reference to the set of `TeloidID`s associated with a given `TeloidTag`.
    ///
    /// # Arguments
    ///
    /// * `tag` - The `TeloidTag` to look up.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `HashSet` of `TeloidID`s if the tag exists,
    /// otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::{TagIndex, TeloidTag, TeloidID};
    /// use std::collections::HashSet;
    ///
    /// let mut tag_index = TagIndex::new();
    /// let tag = "test_tag";
    /// let id = 1;
    /// tag_index.add(tag, id);
    ///
    /// let mut expected = HashSet::new();
    /// expected.insert(1);
    ///
    /// let ids = tag_index.get(tag);
    /// assert_eq!(ids, Some(&expected));
    /// ```
    pub fn get(&self, tag: &str) -> Option<&HashSet<TeloidID>> {
        self.index.get(tag)
    }

    /// Removes a `TeloidID` from the index for a given `TeloidTag`.
    ///
    /// If the tag or ID does not exist, the index remains unchanged.
    ///
    /// # Arguments
    ///
    /// * `tag` - The `TeloidTag` from which to remove the ID.
    /// * `id` - The `TeloidID` to remove.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::{TagIndex, TeloidTag, TeloidID};
    ///
    /// let mut tag_index = TagIndex::new();
    /// let tag = "test_tag";
    /// let id = 1;
    /// tag_index.add(tag, id);
    /// tag_index.remove(tag, id);
    /// assert!(tag_index.get(tag).is_none() || tag_index.get(tag).unwrap().is_empty());
    /// ```
    pub fn remove(&mut self, tag: &str, id: TeloidID) {
        if let Some(set) = self.index.get_mut(tag) {
            set.remove(&id);
            if set.is_empty() {
                self.index.remove(tag);
            }
        }
    }

    /// Adds a `TeloidID` to an existing entry for a given `TeloidTag`.
    ///
    /// This is similar to `add`, but it will not create a new entry if the tag doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `tag` - The `TeloidTag` to update.
    /// * `id` - The `TeloidID` to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::{TagIndex, TeloidTag, TeloidID};
    ///
    /// let mut tag_index = TagIndex::new();
    /// let tag = "test_tag";
    /// let id1 = 1;
    /// let id2 = 2;
    /// tag_index.add(tag, id1);
    /// tag_index.update(tag, id2);
    /// assert_eq!(tag_index.get(tag).unwrap().len(), 2);
    /// ```
    pub fn update(&mut self, tag: &str, id: TeloidID) {
        if let Some(v) = self.index.get_mut(tag) {
            v.insert(id);
        }
    }

    /// Checks if a `TeloidTag` exists in the index.
    ///
    /// # Arguments
    ///
    /// * `tag` - The `TeloidTag` to check.
    ///
    /// # Returns
    ///
    /// `true` if the tag exists, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::{TagIndex, TeloidTag, TeloidID};
    ///
    /// let mut tag_index = TagIndex::new();
    /// let tag = "test_tag";
    /// tag_index.add(tag, 1);
    ///
    /// assert!(tag_index.contains_key(tag));
    /// ```
    pub fn contains_key(&self, tag: &str) -> bool {
        self.index.contains_key(tag)
    }

    /// Returns the number of tags in the index.
    ///
    /// # Returns
    ///
    /// The number of unique tags in the index.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::{TagIndex, TeloidTag, TeloidID};
    ///
    /// let mut tag_index = TagIndex::new();
    /// tag_index.add("test_tag", 1);
    /// assert_eq!(tag_index.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Returns `true` if the index contains no elements.
    ///
    /// # Returns
    ///
    /// `true` if the index is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::TagIndex;
    ///
    /// let mut tag_index = TagIndex::new();
    /// assert!(tag_index.is_empty());
    /// tag_index.add("test_tag", 1);
    /// assert!(!tag_index.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Clears the index, removing all tags and IDs.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_ethos::{TagIndex, TeloidTag, TeloidID};
    ///
    /// let mut tag_index = TagIndex::new();
    /// tag_index.add("test_tag", 1);
    /// tag_index.clear();
    /// assert_eq!(tag_index.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.index.clear();
    }
}
