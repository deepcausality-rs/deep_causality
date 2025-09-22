/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;

use crate::{Observable, ObservableReasoning};

impl<T> ObservableReasoning<T> for [T]
where
    T: Observable,
{
    fn len(&self) -> usize {
        self.len()
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn get_all_items(&self) -> Vec<&T> {
        let mut all: Vec<&T> = Vec::new();
        for item in self {
            all.push(item)
        }
        all
    }
}

impl<K, V> ObservableReasoning<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Observable,
{
    fn len(&self) -> usize {
        self.len()
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn get_all_items(&self) -> Vec<&V> {
        self.values().collect::<Vec<&V>>()
    }
}

impl<K, V> ObservableReasoning<V> for BTreeMap<K, V>
where
    K: Eq + Hash,
    V: Observable,
{
    fn len(&self) -> usize {
        self.len()
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn get_all_items(&self) -> Vec<&V> {
        self.values().collect::<Vec<&V>>()
    }
}

impl<T> ObservableReasoning<T> for Vec<T>
where
    T: Observable,
{
    fn len(&self) -> usize {
        self.len()
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn get_all_items(&self) -> Vec<&T> {
        let mut all: Vec<&T> = Vec::new();
        for item in self {
            all.push(item)
        }
        all
    }
}

impl<T> ObservableReasoning<T> for VecDeque<T>
where
    T: Observable,
{
    fn len(&self) -> usize {
        self.len()
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn get_all_items(&self) -> Vec<&T> {
        let mut all: Vec<&T> = Vec::new();
        for item in self {
            all.push(item)
        }
        all
    }
}
