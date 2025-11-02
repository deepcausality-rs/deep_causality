/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ConstTree;

// Conveniently create a leaf node from a value.
impl<T> From<T> for ConstTree<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
