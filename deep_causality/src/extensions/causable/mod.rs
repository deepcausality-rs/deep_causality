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

use crate::traits::causable::causable_reasoning::CausableReasoning;
use crate::{Causable, IdentificationValue};

impl<K, V> CausableReasoning<V> for HashMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_map_to_vec!();
    make_get_all_map_items!();
    make_find_from_map_values!();
}

impl<K, V> CausableReasoning<V> for BTreeMap<K, V>
where
    K: Eq + Hash,
    V: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_map_to_vec!();
    make_get_all_map_items!();
    make_find_from_map_values!();
}

impl<T> CausableReasoning<T> for [T]
where
    T: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
    make_array_to_vec!();
    make_find_from_iter_values!();
}

impl<T> CausableReasoning<T> for Vec<T>
where
    T: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_vec_to_vec!();
    make_get_all_items!();
    make_find_from_iter_values!();
}

impl<T> CausableReasoning<T> for VecDeque<T>
where
    T: Causable + Clone,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
    make_vec_deq_to_vec!();
    make_find_from_iter_values!();
}
