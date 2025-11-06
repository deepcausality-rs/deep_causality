/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Causable, CausableCollectionAccessor, CausalMonad, MonadicCausable, PropagatingEffect,
};

/// Provides default implementations for monadic reasoning over collections of `MonadicCausable` items.
///
/// Any collection type that implements the basic accessor methods (`len`, `is_empty`,
/// `to_vec`, `get_all_items`) and `MonadicCausable<CausalMonad>` will automatically gain a suite of useful default
/// methods for inspecting the collective state of its `MonadicCausable` elements.
pub trait MonadicCausableCollection<T>: CausableCollectionAccessor<T>
where
    T: MonadicCausable<CausalMonad> + Causable,
{
    //
    // These methods must be implemented by the collection type.
    // See deep_causality/src/extensions/causable/mod.rs
    //

    /// Returns the total number of `MonadicCausable` items in the collection.
    fn len(&self) -> usize;

    /// Checks if the collection of `MonadicCausable` items is empty.
    fn is_empty(&self) -> bool;

    /// Creates a new vector containing the `MonadicCausable` items from the collection.
    fn to_vec(&self) -> Vec<T>;

    /// Evaluates a collection of `MonadicCausable` items, aggregating their monadic effects.
    ///
    /// # Arguments
    /// * `incoming_effect` - A `PropagatingEffect` to be passed to each `MonadicCausable` item.
    ///
    /// # Returns
    /// A `PropagatingEffect` representing the aggregated monadic effect of the collection.
    fn evaluate_collection_monadic(&self, incoming_effect: PropagatingEffect) -> PropagatingEffect;
}
