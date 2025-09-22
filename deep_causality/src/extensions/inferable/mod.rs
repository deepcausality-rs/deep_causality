/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;

use crate::{Inferable, InferableReasoning};

impl<T> InferableReasoning<T> for [T]
where
    T: Inferable,
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

impl<K, V> InferableReasoning<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Inferable,
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

impl<K, V> InferableReasoning<V> for BTreeMap<K, V>
where
    K: Eq + Hash,
    V: Inferable,
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

impl<T> InferableReasoning<T> for Vec<T>
where
    T: Inferable,
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

impl<T> InferableReasoning<T> for VecDeque<T>
where
    T: Inferable,
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
