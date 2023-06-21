/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
// Extension trait http://xion.io/post/code/rust-extension-traits.html
use macros::{make_get_all_items, make_is_empty, make_len, make_vec_to_vec};

use crate::prelude::*;

impl<T> CausableReasoning<T> for Vec<T>
    where
        T: Causable + Clone
{
    make_len!();
    make_is_empty!();
    make_vec_to_vec!();
    make_get_all_items!();
}
