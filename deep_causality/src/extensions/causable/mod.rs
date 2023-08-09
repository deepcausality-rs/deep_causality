// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

// Extension trait http://xion.io/post/code/rust-extension-traits.html
use deep_causality_macros::{make_array_to_vec, make_get_all_items, make_get_all_map_items, make_is_empty, make_len, make_map_to_vec, make_vec_to_vec};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;
use crate::prelude::{Causable, CausableReasoning};

impl<T> CausableReasoning<T> for [T]
    where
        T: Causable + Clone
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
    make_array_to_vec!();
}

impl<K, V> CausableReasoning<V> for BTreeMap<K, V>
    where
        K: Eq + Hash,
        V: Causable + Clone
{
    make_len!();
    make_is_empty!();
    make_map_to_vec!();
    make_get_all_map_items!();
}

impl<K, V> CausableReasoning<V> for HashMap<K, V>
    where
        K: Eq + Hash,
        V: Causable + Clone
{
    make_len!();
    make_is_empty!();
    make_map_to_vec!();
    make_get_all_map_items!();
}

impl<T> CausableReasoning<T> for Vec<T>
    where
        T: Causable + Clone
{
    make_len!();
    make_is_empty!();
    make_vec_to_vec!();
    make_get_all_items!();
}

impl<T> CausableReasoning<T> for VecDeque<T>
    where
        T: Causable + Clone
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
    // VecDeque can't be turned into a vector hence the custom implementation
    // https://github.com/rust-lang/rust/issues/23308
    // Also, make_contiguous requires self to be mutable, which would violate the API.
    // https://doc.rust-lang.org/std/collections/struct.VecDeque.html#method.make_contiguous
    fn to_vec(&self) -> Vec<T> {
        let mut v = Vec::with_capacity(self.len());
        let mut deque = self.clone(); // clone to avoid mutating the original

        for item in deque.make_contiguous().iter() {
            v.push(item.clone());
        }

        v
    }
}
