/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;

// Extension trait http://xion.io/post/code/rust-extension-traits.html
use deep_causality_macros::{
    make_array_to_vec, make_find_from_iter_values, make_find_from_map_values, make_get_all_items,
    make_get_all_map_items, make_is_empty, make_len, make_map_to_vec, make_vec_deq_to_vec,
    make_vec_to_vec,
};

use crate::{
    Causable, CausableCollection, CausableCollectionAccessor, CausableCollectionExplaining,
    CausableCollectionReasoning, IdentificationValue,
};

//
// HashMap<K, V>
//
impl<K, V> CausableCollection<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
}

impl<K, V> CausableCollectionExplaining<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
}

impl<K, V> CausableCollectionAccessor<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
    make_get_all_map_items!();
}

impl<K, V> CausableCollectionReasoning<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_map_to_vec!();
    make_find_from_map_values!();
}

//
// BTreeMap<K, V>
//
impl<K, V> CausableCollection<V> for BTreeMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
}

impl<K, V> CausableCollectionExplaining<V> for BTreeMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
}

impl<K, V> CausableCollectionAccessor<V> for BTreeMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
    make_get_all_map_items!();
}

impl<K, V> CausableCollectionReasoning<V> for BTreeMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_map_to_vec!();
    make_find_from_map_values!();
}

//
// [T]
//
impl<T> CausableCollection<T> for [T] where T: Causable + Clone {}

impl<T> CausableCollectionExplaining<T> for [T] where T: Causable + Clone {}

impl<T> CausableCollectionAccessor<T> for [T]
where
    T: Causable + Clone,
{
    make_get_all_items!();
}

impl<T> CausableCollectionReasoning<T> for [T]
where
    T: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_array_to_vec!();
    make_find_from_iter_values!();
}

//
//  Vec<T>
//
impl<T> CausableCollection<T> for Vec<T> where T: Causable + Clone {}

impl<T> CausableCollectionExplaining<T> for Vec<T> where T: Causable + Clone {}

impl<T> CausableCollectionAccessor<T> for Vec<T>
where
    T: Causable + Clone,
{
    make_get_all_items!();
}

impl<T> CausableCollectionReasoning<T> for Vec<T>
where
    T: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_vec_to_vec!();
    make_find_from_iter_values!();
}

//
//  VecDeque
//
impl<T> CausableCollection<T> for VecDeque<T> where T: Causable + Clone {}

impl<T> CausableCollectionExplaining<T> for VecDeque<T> where T: Causable + Clone {}

impl<T> CausableCollectionAccessor<T> for VecDeque<T>
where
    T: Causable + Clone,
{
    make_get_all_items!();
}

impl<T> CausableCollectionReasoning<T> for VecDeque<T>
where
    T: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_vec_deq_to_vec!();
    make_find_from_iter_values!();
}
