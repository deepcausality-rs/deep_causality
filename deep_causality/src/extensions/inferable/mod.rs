/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
// Extension trait http://xion.io/post/code/rust-extension-traits.html

use deep_causality_macros::{make_get_all_items, make_get_all_map_items, make_is_empty, make_len};
use std::collections::HashMap;
use std::hash::Hash;
use crate::prelude::{Inferable, InferableReasoning};


impl<T> InferableReasoning<T> for Vec<T>
    where
        T: Inferable,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}


impl<K, V> InferableReasoning<V> for HashMap<K, V>
    where
        K: Eq + Hash,
        V: Inferable + Clone,
{
    make_len!();
    make_is_empty!();
    make_get_all_map_items!();
}


impl<T> InferableReasoning<T> for [T]
    where
        T: Inferable,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}
