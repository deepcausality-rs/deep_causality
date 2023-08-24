// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::Debug;

use crate::prelude::{Causable, CausaloidGraph};

impl<T> Default for CausaloidGraph<T>
where
    T: Debug + Causable + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}
