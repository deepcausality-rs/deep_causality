/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */


// Extension trait http://xion.io/post/code/rust-extension-traits.html
use deep_causality_macros::{make_array_to_vec, make_get_all_items, make_get_all_map_items, make_is_empty, make_len, make_map_to_vec, make_vec_to_vec};
use std::collections::HashMap;
use std::hash::Hash;
use crate::prelude::{Causable, CausableReasoning};


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

impl<T> CausableReasoning<T> for [T]
    where
        T: Causable + Clone
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
    make_array_to_vec!();
}
