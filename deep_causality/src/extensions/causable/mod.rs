/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Extension trait http://xion.io/post/code/rust-extension-traits.html
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;

use crate::{
    Causable, CausableCollectionAccessor, Identifiable, MonadicCausable, MonadicCausableCollection,
};
use crate::{CausalMonad, IdentificationValue};

//
// [T]
//
impl<T> MonadicCausableCollection<T> for [T] where T: MonadicCausable<CausalMonad> + Clone + Causable
{}

impl<T> CausableCollectionAccessor<T> for [T]
where
    T: MonadicCausable<CausalMonad> + Clone + Identifiable,
{
    fn get_all_items(&self) -> Vec<&T> {
        self.iter().collect()
    }

    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn to_vec(&self) -> Vec<T> {
        self.to_vec()
    }

    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T> {
        self.iter().find(|&item| item.id() == id)
    }
}

//
//  VecDeque
//
impl<T> MonadicCausableCollection<T> for VecDeque<T> where
    T: MonadicCausable<CausalMonad> + Causable + Clone
{
}

impl<T> CausableCollectionAccessor<T> for VecDeque<T>
where
    T: MonadicCausable<CausalMonad> + Clone + Identifiable,
{
    fn get_all_items(&self) -> Vec<&T> {
        self.iter().collect()
    }

    fn len(&self) -> usize {
        VecDeque::len(self)
    }

    fn is_empty(&self) -> bool {
        VecDeque::is_empty(self)
    }

    fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T> {
        self.iter().find(|&item| item.id() == id)
    }
}

//
// HashMap<K, V>
//
impl<K, V> MonadicCausableCollection<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: MonadicCausable<CausalMonad> + Causable + Clone,
{
}

impl<K, V> CausableCollectionAccessor<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: MonadicCausable<CausalMonad> + Clone + Identifiable,
{
    fn get_all_items(&self) -> Vec<&V> {
        self.values().collect::<Vec<&V>>()
    }

    fn len(&self) -> usize {
        HashMap::len(self)
    }

    fn is_empty(&self) -> bool {
        HashMap::is_empty(self)
    }

    fn to_vec(&self) -> Vec<V> {
        self.values().cloned().collect()
    }

    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&V> {
        self.values().find(|&item| item.id() == id)
    }
}

//
// BTreeMap<K, V>
//
impl<K, V> MonadicCausableCollection<V> for BTreeMap<K, V>
where
    K: Ord,
    V: MonadicCausable<CausalMonad> + Causable + Clone,
{
}

impl<K, V> CausableCollectionAccessor<V> for BTreeMap<K, V>
where
    K: Ord,
    V: MonadicCausable<CausalMonad> + Causable + Clone + Identifiable,
{
    fn get_all_items(&self) -> Vec<&V> {
        self.values().collect::<Vec<&V>>()
    }

    fn len(&self) -> usize {
        BTreeMap::len(self)
    }

    fn is_empty(&self) -> bool {
        BTreeMap::is_empty(self)
    }

    fn to_vec(&self) -> Vec<V> {
        self.values().cloned().collect()
    }

    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&V> {
        self.values().find(|&item| item.id() == id)
    }
}
