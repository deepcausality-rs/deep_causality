// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality_macros::{make_get_all_items, make_get_all_map_items, make_is_empty, make_len};
use std::collections::HashMap;
use std::hash::Hash;
use crate::prelude::{Assumable, AssumableReasoning};


impl<T> AssumableReasoning<T> for [T]
    where
        T: Assumable + Clone,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}


impl<K, V> AssumableReasoning<V> for HashMap<K, V>
    where
        K: Eq + Hash,
        V: Assumable + Clone,
{
    make_len!();
    make_is_empty!();
    make_get_all_map_items!();
}


impl<T> AssumableReasoning<T> for Vec<T>
    where
        T: Assumable + Clone,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}
