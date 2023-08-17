// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
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
