/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;

// Extension trait http://xion.io/post/code/rust-extension-traits.html
use crate::{
    Causable, CausableCollection, CausableCollectionAccessor, CausableCollectionExplaining,
    CausalMonad, IdentificationValue, MonadicCausable, MonadicCausableCollection,
    PropagatingEffect,
};

//
// [T]
//
impl<T> CausableCollection<T> for [T] where T: MonadicCausable<CausalMonad> + Clone + Causable {}

impl<T> CausableCollectionExplaining<T> for [T] where
    T: MonadicCausable<CausalMonad> + Causable + Clone
{
}

impl<T> CausableCollectionAccessor<T> for [T]
where
    T: MonadicCausable<CausalMonad> + Clone,
{
    fn get_all_items(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

//
//  Vec<T>
//
impl<T> CausableCollection<T> for Vec<T> where T: MonadicCausable<CausalMonad> + Causable + Clone {}

impl<T> CausableCollectionExplaining<T> for Vec<T> where
    T: MonadicCausable<CausalMonad> + Causable + Clone
{
}

impl<T> CausableCollectionAccessor<T> for Vec<T>
where
    T: MonadicCausable<CausalMonad> + Clone,
{
    fn get_all_items(&self) -> Vec<&T> {
        self.iter().collect()
    }
}

impl<T> MonadicCausableCollection<T> for Vec<T>
where
    T: MonadicCausable<CausalMonad> + Clone + Causable,
{
    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn to_vec(&self) -> Vec<T> {
        self.clone()
    }

    fn evaluate_collection_monadic(&self, incoming_effect: PropagatingEffect) -> PropagatingEffect {
        let mut effects: HashMap<IdentificationValue, Box<PropagatingEffect>> = HashMap::new();
        for item in self {
            effects.insert(
                item.id(),
                Box::new(item.evaluate_monadic(incoming_effect.clone())),
            );
        }
        PropagatingEffect::from_map(effects)
    }
}

//
//  VecDeque
//
impl<T> CausableCollection<T> for VecDeque<T> where
    T: MonadicCausable<CausalMonad> + Causable + Clone
{
}

impl<T> CausableCollectionExplaining<T> for VecDeque<T> where
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
impl<K, V> CausableCollection<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: MonadicCausable<CausalMonad> + Causable + Clone,
{
}

impl<K, V> CausableCollectionExplaining<V> for HashMap<K, V>
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
impl<K, V> CausableCollection<V> for BTreeMap<K, V>
where
    K: Ord,
    V: MonadicCausable<CausalMonad> + Causable + Clone,
{
}

impl<K, V> CausableCollectionExplaining<V> for BTreeMap<K, V>
where
    K: Ord,
    V: MonadicCausable<CausalMonad> + Clone,
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
