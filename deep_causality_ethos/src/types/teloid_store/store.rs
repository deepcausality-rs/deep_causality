/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};

use crate::{Teloid, TeloidID, TeloidStorable, TeloidStore};

impl<D, S, T, ST, SYM, VS, VT> TeloidStorable<D, S, T, ST, SYM, VS, VT>
    for TeloidStore<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
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
    fn insert(
        &mut self,
        teloid: Teloid<D, S, T, ST, SYM, VS, VT>,
    ) -> Option<Teloid<D, S, T, ST, SYM, VS, VT>> {
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
    fn get(&self, id: &TeloidID) -> Option<&Teloid<D, S, T, ST, SYM, VS, VT>> {
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
    fn remove(&mut self, id: &TeloidID) -> Option<Teloid<D, S, T, ST, SYM, VS, VT>> {
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
    fn update(
        &mut self,
        teloid: Teloid<D, S, T, ST, SYM, VS, VT>,
    ) -> Option<Teloid<D, S, T, ST, SYM, VS, VT>> {
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
    fn contains_key(&self, id: &TeloidID) -> bool {
        self.index.contains_key(id)
    }
    /// Returns the number of `Teloid`s in the store.
    ///
    /// # Returns
    ///
    /// The number of `Teloid`s in the store.
    ///
    fn len(&self) -> usize {
        self.index.len()
    }
    /// Returns `true` if the store contains no `Teloid`s.
    ///
    /// # Returns
    ///
    /// `true` if the store is empty, otherwise `false`.
    ///
    fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
    /// Clears the store, removing all `Teloid`s.
    ///
    fn clear(&mut self) {
        self.index.clear()
    }
}
