/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Causable, CausaloidGraph, Identifiable};
use std::fmt::Display;

impl<T> Identifiable for CausaloidGraph<T>
where
    T: Causable + Clone + Display + PartialEq,
{
    fn id(&self) -> u64 {
        self.id
    }
}
