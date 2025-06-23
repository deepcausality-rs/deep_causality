// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::Display;

use crate::prelude::{Causable, CausaloidGraph};

impl<T> Default for CausaloidGraph<T>
where
    T: Clone + Display + Causable + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}
