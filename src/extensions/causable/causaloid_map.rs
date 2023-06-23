/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
// Extension trait http://xion.io/post/code/rust-extension-traits.html

use std::collections::HashMap;
use std::hash::Hash;

use macros::{make_get_all_map_items, make_is_empty, make_len, make_map_to_vec};
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
