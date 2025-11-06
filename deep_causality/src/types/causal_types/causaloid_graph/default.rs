/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalMonad, CausaloidGraph, MonadicCausable};
use std::fmt::Display;

impl<T> Default for CausaloidGraph<T>
where
    T: MonadicCausable<CausalMonad> + PartialEq + Clone + Display,
{
    fn default() -> Self {
        Self::new(0)
    }
}
