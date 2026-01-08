/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Teloid, TeloidID};
use deep_causality::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
pub trait TeloidStorable<D, S, T, ST, SYM, VS, VT>
where
    D: Clone + Datable,
    S: Clone + Spatial<VS>,
    ST: Clone + SpaceTemporal<VS, VT>,
    SYM: Clone + Symbolic,
    T: Clone + Temporal<VT>,
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
    ) -> Option<Teloid<D, S, T, ST, SYM, VS, VT>>;
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
    fn get(&self, id: &TeloidID) -> Option<&Teloid<D, S, T, ST, SYM, VS, VT>>;
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
    fn remove(&mut self, id: &TeloidID) -> Option<Teloid<D, S, T, ST, SYM, VS, VT>>;
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
    ) -> Option<Teloid<D, S, T, ST, SYM, VS, VT>>;
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
    fn contains_key(&self, id: &TeloidID) -> bool;
    /// Returns the number of `Teloid`s in the store.
    ///
    /// # Returns
    ///
    /// The number of `Teloid`s in the store.
    ///
    fn len(&self) -> usize;
    /// Returns `true` if the store contains no `Teloid`s.
    ///
    /// # Returns
    ///
    /// `true` if the store is empty, otherwise `false`.
    ///
    fn is_empty(&self) -> bool;
    /// Clears the store, removing all `Teloid`s.
    ///
    fn clear(&mut self);
}
