// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{Contextoid, Datable, Identifiable, SpaceTemporal, Spatial, Temporal};

impl<D, S, T, ST> Identifiable for Contextoid<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    fn id(&self) -> u64 {
        self.id
    }
}
