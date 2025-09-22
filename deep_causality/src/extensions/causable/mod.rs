/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;

// Extension trait http://xion.io/post/code/rust-extension-traits.html
use crate::{
    Causable, CausableCollection, CausableCollectionAccessor, CausableCollectionExplaining,
    CausableCollectionReasoning, IdentificationValue,
};

//
// [T]
//
impl<T> CausableCollection<T> for [T] where T: Causable + Clone {}

impl<T> CausableCollectionExplaining<T> for [T] where T: Causable + Clone {}

impl<T> CausableCollectionAccessor<T> for [T]
where
    T: Causable + Clone,
{
    fn get_all_items(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T> CausableCollectionReasoning<T> for [T]
where
    T: Causable + Clone,
{
    fn len(&self) -> usize {
        <[T]>::len(self)
    }

    fn is_empty(&self) -> bool {
        <[T]>::is_empty(self)
    }
    fn to_vec(&self) -> Vec<T> {
        self.to_vec()
    }
    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T> {
        self.iter().find(|item| item.id() == id)
    }
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
    fn get_all_items(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T> CausableCollectionReasoning<T> for Vec<T>
where
    T: Causable + Clone,
{
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
    fn to_vec(&self) -> Vec<T> {
        self.clone()
    }
    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T> {
        self.iter().find(|item| item.id() == id)
    }
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
    fn get_all_items(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T> CausableCollectionReasoning<T> for VecDeque<T>
where
    T: Causable + Clone,
{
    fn len(&self) -> usize {
        VecDeque::len(self)
    }
    fn is_empty(&self) -> bool {
        VecDeque::is_empty(self)
    }
    fn to_vec(&self) -> Vec<T> {
        let mut v = Vec::with_capacity(self.len());
        let mut deque = self.clone(); // clone to avoid mutating the original

        for item in deque.make_contiguous().iter() {
            v.push(item.clone());
        }

        v
    }
    fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T> {
        self.iter().find(|item| item.id() == id)
    }
}

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
    fn get_all_items(&self) -> Vec<&V> {
        self.values().collect::<Vec<&V>>()
    }
}

impl<K, V> CausableCollectionReasoning<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
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
        self.values().find(|item| item.id() == id)
    }
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
    fn get_all_items(&self) -> Vec<&V> {
        self.values().collect::<Vec<&V>>()
    }
}

impl<K, V> CausableCollectionReasoning<V> for BTreeMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
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
        self.values().find(|item| item.id() == id)
    }
}
