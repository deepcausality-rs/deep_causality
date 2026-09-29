/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The map and set the context keys by id or index.
//!
//! Under `std` they are `HashMap` and `HashSet`: a context can hold tens of thousands of nodes, and
//! at 10 000 keys a `BTreeMap` lookup costs about five times a hashed one. Without `std` there is no
//! `RandomState` to seed the default hasher, so they are `BTreeMap` and `BTreeSet`. Every key is an
//! integer id or a tuple of them, which satisfies both `Hash + Eq` and `Ord`. Iteration order is
//! unspecified under `std` and sorted without it; no caller depends on it.

#[cfg(feature = "std")]
pub(crate) type IdMap<K, V> = std::collections::HashMap<K, V>;
#[cfg(not(feature = "std"))]
pub(crate) type IdMap<K, V> = alloc::collections::BTreeMap<K, V>;

#[cfg(feature = "std")]
pub(crate) type IdSet<T> = std::collections::HashSet<T>;
#[cfg(not(feature = "std"))]
pub(crate) type IdSet<T> = alloc::collections::BTreeSet<T>;

/// An empty [`IdMap`] with room for `capacity` entries where the map type can reserve it.
#[cfg(feature = "std")]
pub(crate) fn id_map_with_capacity<K, V>(capacity: usize) -> IdMap<K, V> {
    IdMap::with_capacity(capacity)
}

/// An empty [`IdMap`]; a `BTreeMap` has no capacity to reserve.
#[cfg(not(feature = "std"))]
pub(crate) fn id_map_with_capacity<K, V>(_capacity: usize) -> IdMap<K, V> {
    IdMap::new()
}

/// An empty [`IdSet`] with room for `capacity` entries where the set type can reserve it.
#[cfg(feature = "std")]
pub(crate) fn id_set_with_capacity<T>(capacity: usize) -> IdSet<T> {
    IdSet::with_capacity(capacity)
}

/// An empty [`IdSet`]; a `BTreeSet` has no capacity to reserve.
#[cfg(not(feature = "std"))]
pub(crate) fn id_set_with_capacity<T>(_capacity: usize) -> IdSet<T> {
    IdSet::new()
}
