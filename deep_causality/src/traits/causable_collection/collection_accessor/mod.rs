/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalMonad, IdentificationValue, MonadicCausable};

pub trait CausableCollectionAccessor<T>
where
    T: MonadicCausable<CausalMonad>,
{
    //
    // All these methods must be implemented by the collection type.
    // See deep_causality/src/extensions/causable/mod.rs
    //

    /// Returns a vector of references to all `Causable` items in the collection.
    /// This is the primary accessor used by the trait's default methods.
    fn get_all_items(&self) -> Vec<&T>;

    /// Returns the total number of `Causable` items in the collection.
    fn len(&self) -> usize;

    /// Checks if the collection of `Causable` items is empty.
    fn is_empty(&self) -> bool;

    /// Creates a new vector containing the `Causable` items from the collection.
    fn to_vec(&self) -> Vec<T>;

    /// Returns a reference to a `Causable` item by its ID, if found.
    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T>;
}
