// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use crate::prelude::{Context, Datable, Identifiable, SpaceTemporal, Spatial, Temporal};

impl<'l, D, S, T, ST> Identifiable for Context<'l, D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    /// Returns the id of the context.
    fn id(&self) -> u64 {
        self.id
    }
}
