/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Identifiable, Teloid, TeloidID};
use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TeloidStore {
    index: HashMap<TeloidID, Teloid>,
}

impl TeloidStore {
    /// Creates a new, empty `TeloidStore`.
    ///
    /// # Returns
    ///
    /// A new `TeloidStore` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::TeloidStore;
    ///
    /// let store = TeloidStore::new();
    /// assert!(store.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Creates a new `TeloidStore` with a specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The initial capacity of the store.
    ///
    /// # Returns
    ///
    /// A new `TeloidStore` instance with the given capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::TeloidStore;
    ///     
    /// let store = TeloidStore::with_capacity(10);
    /// assert!(store.is_empty());
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
        }
    }
}

impl TeloidStore {
    /// Inserts a `Teloid` into the store.
    ///
    /// If the store did not have this ID present, `None` is returned.
    /// If the store did have this ID present, the value is updated, and the old
    /// value is returned.
    ///
    /// # Arguments
    ///
    /// * `teloid` - The `Teloid` to insert.
    ///
    /// # Returns
    ///
    /// An `Option` containing the old `Teloid` if the ID already existed, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{Identifiable, Teloid, TeloidModal, TeloidStore};
    ///
    /// let mut store = TeloidStore::new();
    /// let teloid = Teloid::new(1, Vec::new() ,TeloidModal::Obligatory , None);
    /// store.insert(teloid);
    /// assert!(store.contains_key(&1));
    /// ```
    pub fn insert(&mut self, teloid: Teloid) -> Option<Teloid> {
        self.index.insert(teloid.id(), teloid)
    }

    /// Retrieves a reference to a `Teloid` from the store.
    ///
    /// # Arguments
    ///
    /// * `id` - The `TeloidID` of the `Teloid` to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `Teloid` if it exists, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{Identifiable, Teloid, TeloidModal, TeloidStore};
    ///
    /// let mut store = TeloidStore::new();
    /// let teloid = Teloid::new(1, Vec::new() ,TeloidModal::Obligatory , None);
    /// store.insert(teloid.clone());
    ///
    /// let retrieved = store.get(&teloid.id());
    /// assert_eq!(retrieved, Some(&teloid));
    /// ```
    pub fn get(&self, id: &TeloidID) -> Option<&Teloid> {
        self.index.get(id)
    }

    /// Removes a `Teloid` from the store, returning it.
    ///
    /// # Arguments
    ///
    /// * `id` - The `TeloidID` of the `Teloid` to remove.
    ///
    /// # Returns
    ///
    /// An `Option` containing the removed `Teloid` if it existed, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{Identifiable, Teloid, TeloidModal, TeloidStore};
    ///
    /// let mut store = TeloidStore::new();
    /// let teloid = Teloid::new(1, Vec::new() ,TeloidModal::Obligatory , None);
    /// store.insert(teloid.clone());
    ///
    /// let removed = store.remove(&teloid.id());
    /// assert_eq!(removed, Some(teloid));
    /// assert!(!store.contains_key(&1));
    /// ```
    pub fn remove(&mut self, id: &TeloidID) -> Option<Teloid> {
        self.index.remove(id)
    }

    /// Updates a `Teloid` in the store. This is an alias for `insert`.
    ///
    /// If the store did not have this ID present, `None` is returned.
    /// If the store did have this ID present, the value is updated, and the old
    /// value is returned.
    ///
    /// # Arguments
    ///
    /// * `teloid` - The `Teloid` to insert/update.
    ///
    /// # Returns
    ///
    /// An `Option` containing the old `Teloid` if the ID already existed, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{Teloid , TeloidModal, TeloidStore};
    ///
    /// let mut store = TeloidStore::new();
    /// let teloid1 = Teloid::new(1, Vec::new() ,TeloidModal::Obligatory , None);
    /// store.insert(teloid1);
    ///
    /// let teloid2 = Teloid::new(1, Vec::new() ,TeloidModal::Obligatory , None);
    /// store.update(teloid2.clone());
    ///
    /// assert_eq!(store.get(&1), Some(&teloid2));
    /// ```
    pub fn update(&mut self, teloid: Teloid) -> Option<Teloid> {
        self.index.insert(teloid.id(), teloid)
    }

    /// Checks if the store contains a `Teloid` with the specified ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The `TeloidID` to check for.
    ///
    /// # Returns
    ///
    /// `true` if the store contains the ID, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{Teloid , TeloidModal, TeloidStore};
    ///
    /// let mut store = TeloidStore::new();
    /// let teloid = Teloid::new(1, Vec::new() ,TeloidModal::Obligatory , None);
    /// store.insert(teloid);
    ///
    /// assert!(store.contains_key(&1));
    /// assert!(!store.contains_key(&2));
    /// ```
    pub fn contains_key(&self, id: &TeloidID) -> bool {
        self.index.contains_key(id)
    }

    /// Returns the number of `Teloid`s in the store.
    ///
    /// # Returns
    ///
    /// The number of `Teloid`s in the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{Teloid , TeloidModal, TeloidStore};
    ///
    /// let mut store = TeloidStore::new();
    /// assert_eq!(store.len(), 0);
    ///
    /// let teloid = Teloid::new(1, Vec::new() ,TeloidModal::Obligatory , None);
    /// store.insert(teloid);
    /// assert_eq!(store.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Returns `true` if the store contains no `Teloid`s.
    ///
    /// # Returns
    ///
    /// `true` if the store is empty, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{Teloid , TeloidModal, TeloidStore};
    ///
    /// let mut store = TeloidStore::new();
    /// assert!(store.is_empty());
    ///
    /// let teloid = Teloid::new(1, Vec::new() ,TeloidModal::Obligatory , None);
    /// store.insert(teloid);
    /// assert!(!store.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Clears the store, removing all `Teloid`s.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{Teloid , TeloidModal, TeloidStore};
    ///
    /// let mut store = TeloidStore::new();
    /// let teloid = Teloid::new(1, Vec::new() ,TeloidModal::Obligatory , None);
    /// store.insert(teloid);
    /// store.clear();
    /// assert!(store.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.index.clear()
    }
}
