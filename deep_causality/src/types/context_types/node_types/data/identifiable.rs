// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::Identifiable;
use crate::types::context_types::node_types::data::Dataoid;

impl<T> Identifiable for Dataoid<T>
where
    T: Copy + Default,
{
    fn id(&self) -> u64 {
        self.id
    }
}
