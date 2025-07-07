/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::Display;

use crate::{Causable, CausaloidGraph};

impl<T> Default for CausaloidGraph<T>
where
    T: Clone + Display + Causable + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}
