// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality_macros::{make_get_all_items, make_get_all_map_items, make_is_empty, make_len};
use std::collections::{ BTreeMap, HashMap, VecDeque};
use std::hash::Hash;
use crate::prelude::{Assumable, AssumableReasoning};


/// https://doc.rust-lang.org/std/primitive.array.html
impl<T> AssumableReasoning<T> for [T]
    where
        T: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}

/// https://doc.rust-lang.org/std/collections/hash_map/struct.HashMap.html
impl<K, V> AssumableReasoning<V> for HashMap<K, V>
    where
        K: Eq + Hash,
        V: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_map_items!();
}

/// https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
impl<K, V> AssumableReasoning<V> for BTreeMap<K, V>
    where
        K: Eq + Hash,
        V: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_map_items!();
}

/// https://doc.rust-lang.org/std/vec/struct.Vec.html
impl<T> AssumableReasoning<T> for Vec<T>
    where
        T: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}

/// https://doc.rust-lang.org/std/collections/struct.VecDeque.html
impl<T> AssumableReasoning<T> for VecDeque<T>
    where
        T: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}
