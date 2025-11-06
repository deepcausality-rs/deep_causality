/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;

// Extension trait http://xion.io/post/code/rust-extension-traits.html
use crate::{
    Causable, CausableCollectionAccessor, CausalMonad, MonadicCausable, MonadicCausableCollection,
};

//
// [T]
//
impl<T> MonadicCausableCollection<T> for [T] where T: MonadicCausable<CausalMonad> + Clone + Causable
{}

impl<T> CausableCollectionAccessor<T> for [T]
where
    T: MonadicCausable<CausalMonad> + Clone,
{
    fn get_all_items(&self) -> Vec<&T> {
        self.iter().collect()
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
    T: MonadicCausable<CausalMonad> + Clone,
{
    fn get_all_items(&self) -> Vec<&T> {
        self.iter().collect()
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
    V: MonadicCausable<CausalMonad> + Clone,
{
    fn get_all_items(&self) -> Vec<&V> {
        self.values().collect::<Vec<&V>>()
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
    V: MonadicCausable<CausalMonad> + Clone,
{
    fn get_all_items(&self) -> Vec<&V> {
        self.values().collect::<Vec<&V>>()
    }
}
