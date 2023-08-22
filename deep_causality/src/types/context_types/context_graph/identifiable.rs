// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{Context, Datable, Identifiable, SpaceTemporal, Spatial, Temporable};

impl<'l, D, S, T, ST> Identifiable for Context<'l, D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporable,
        ST: SpaceTemporal,
{
    /// Returns the id of the context.
    fn id(&self) -> u64 {
        self.id
    }
}
