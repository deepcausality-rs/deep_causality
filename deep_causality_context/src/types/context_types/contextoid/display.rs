/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt::{Display, Formatter};

use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::{Contextoid, Datable};

impl<D, S, T, ST> Display for Contextoid<D, S, T, ST>
where
    D: Datable + Clone + Display,
    S: Spatial + Clone + Display,
    T: Temporal + Clone + Display,
    ST: SpaceTemporal + Clone + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Contextoid ID: {} Type: {}", self.id, self.vertex_type)
    }
}
