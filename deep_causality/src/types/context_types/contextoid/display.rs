// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use std::fmt::{Display, Formatter};

use crate::prelude::{Contextoid, Datable, SpaceTemporal, Spatial, Temporal};

impl<D, S, T, ST> Display for Contextoid<D, S, T, ST>
    where
        D: Datable + Display,
        S: Spatial + Display,
        T: Temporal + Display,
        ST: SpaceTemporal + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Contextoid ID: {} Type: {}",
               self.id,
               self.vertex_type
        )
    }
}
