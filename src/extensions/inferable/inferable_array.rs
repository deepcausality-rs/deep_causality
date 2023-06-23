/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
// Extension trait http://xion.io/post/code/rust-extension-traits.html

use macros::{make_get_all_items, make_is_empty, make_len};
use crate::prelude::{Inferable, InferableReasoning};


impl<T> InferableReasoning<T> for [T]
    where
        T: Inferable,
{
    make_len!();
    make_is_empty!();
    make_get_all_items!();
}