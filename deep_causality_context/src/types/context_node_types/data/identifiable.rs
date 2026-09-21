/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::context_node_types::data::Data;
use deep_causality_core::Identifiable;

impl<T> Identifiable for Data<T>
where
    T: Default + Clone + PartialEq,
{
    fn id(&self) -> u64 {
        self.id
    }
}
