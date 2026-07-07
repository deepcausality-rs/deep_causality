/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The labeled parent-effect map delivered to a reconvergence node's join mechanism.
//!
//! At a fan-in the reasoning engine gathers the effects of the parents that actually
//! *fired* (as opposed to those whose wire resolved `Inactive`), keyed by the parent's
//! node index, and hands them to the node's declared join mechanism as a
//! [`ParentEffects`]. The keying is what makes fan-in the *labeled* monoidal product
//! `⊗_{p ∈ Pa(n)} X_p` — parent identity is preserved, so an asymmetric mechanism
//! `f(x_1, x_2) ≠ f(x_2, x_1)` and Pearl do-surgery on a single edge both have a
//! stable handle. Determinism follows from the keys, not from any algebraic property
//! of the mechanism: the underlying `BTreeMap` iterates in ascending parent-index
//! order regardless of the order in which parents fired.

use crate::PropagatingEffect;
use std::collections::BTreeMap;

/// The fired parent effects at a reconvergence node, keyed by parent node index.
///
/// Only parents that fired are present; a parent whose wire resolved `Inactive`
/// contributes no entry. Iteration is always in ascending parent-index order.
///
/// # Type Parameters
/// - `V`: the effect value type flowing through the graph.
pub struct ParentEffects<V> {
    /// Fired parents keyed by parent node index (canonical ascending iteration order).
    fired: BTreeMap<usize, PropagatingEffect<V>>,
}

impl<V> ParentEffects<V> {
    /// Wraps a map of fired parent effects (keyed by parent node index).
    #[inline]
    pub fn new(fired: BTreeMap<usize, PropagatingEffect<V>>) -> Self {
        ParentEffects { fired }
    }

    /// Returns the effect of parent `parent_index`, if that parent fired.
    #[inline]
    pub fn get(&self, parent_index: usize) -> Option<&PropagatingEffect<V>> {
        self.fired.get(&parent_index)
    }

    /// Iterates the fired parents as `(parent_index, effect)` in ascending index order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (usize, &PropagatingEffect<V>)> {
        self.fired.iter().map(|(k, v)| (*k, v))
    }

    /// Iterates the fired parent node indices in ascending order.
    #[inline]
    pub fn parent_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.fired.keys().copied()
    }

    /// The number of fired parents.
    #[inline]
    pub fn len(&self) -> usize {
        self.fired.len()
    }

    /// Whether no parent fired.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fired.is_empty()
    }
}
