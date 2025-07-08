/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::hash::Hash;

use deep_causality_macros::{make_get_all_items, make_get_all_map_items, make_is_empty, make_len};

use crate::{Assumable, AssumableReasoning};

impl<T> AssumableReasoning<T> for [T]
where
    T: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}

impl<K, V> AssumableReasoning<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_map_items!();
}

impl<K, V> AssumableReasoning<V> for BTreeMap<K, V>
where
    K: Eq + Hash,
    V: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_map_items!();
}

impl<T> AssumableReasoning<T> for Vec<T>
where
    T: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}

impl<T> AssumableReasoning<T> for VecDeque<T>
where
    T: Assumable,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}
