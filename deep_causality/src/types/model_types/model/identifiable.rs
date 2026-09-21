/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Model;
use deep_causality_core::Identifiable;
use std::fmt::Debug;

#[allow(clippy::type_complexity)]
impl<I, O, C> Identifiable for Model<I, O, C>
where
    I: Default,
    O: Default + Debug,
    C: Clone,
{
    fn id(&self) -> u64 {
        self.id
    }
}
