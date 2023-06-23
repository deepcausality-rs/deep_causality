/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
// Extension trait http://xion.io/post/code/rust-extension-traits.html
use macros::{make_array_to_vec, make_get_all_items, make_is_empty, make_len};
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